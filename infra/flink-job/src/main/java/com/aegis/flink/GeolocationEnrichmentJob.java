package com.aegis.flink;

import aegis.types.EnrichedHost;
import aegis.types.L4Response;
import org.apache.flink.api.common.eventtime.WatermarkStrategy;
import org.apache.flink.api.common.serialization.DeserializationSchema;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.connector.kafka.sink.KafkaRecordSerializationSchema;
import org.apache.flink.connector.kafka.sink.KafkaSink;
import org.apache.flink.connector.kafka.source.KafkaSource;
import org.apache.flink.connector.kafka.source.enumerator.initializer.OffsetsInitializer;
import org.apache.flink.streaming.api.CheckpointingMode;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.apache.flink.streaming.api.functions.ProcessFunction;
import org.apache.flink.streaming.api.functions.async.AsyncFunction;
import org.apache.flink.streaming.api.functions.async.ResultFuture;
import org.apache.flink.util.Collector;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.time.Instant;
import java.util.Base64;
import java.util.Collections;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * AEGIS Tier 1 Geolocation Enrichment Flink Job
 *
 * Real-time streaming enrichment for Tier 1 geolocation (cryptographic geofeed
 * validation via RPKI ROAs per RFC 9092/9632/8805).
 *
 * Pipeline:
 * 1. Read L4Response from Kafka topic raw-l4-scans
 * 2. For each IP, fetch matching geofeed entry from Tier 1 cache (RPKI-validated)
 * 3. Validate ROA authorization for the prefix/origin AS
 * 4. Emit EnrichedHost with Tier 1 geolocation to enrichment-events topic
 *
 * Performance targets:
 * - End-to-end latency: < 5s
 * - Throughput: 10K events/s
 */
public class GeolocationEnrichmentJob {

    private static final Logger LOG = LoggerFactory.getLogger(GeolocationEnrichmentJob.class);

    // Configuration
    private static final String KAFKA_BROKER = System.getenv().getOrDefault("KAFKA_BROKER", "aegis-kafka:9092");
    private static final String INPUT_TOPIC = System.getenv().getOrDefault("INPUT_TOPIC", "raw-l4-scans");
    private static final String OUTPUT_TOPIC = System.getenv().getOrDefault("OUTPUT_TOPIC", "enrichment-events");
    private static final String GEOFEED_API_BASE = System.getenv().getOrDefault("GEOFEED_API_BASE", "http://aegis-enrichment:8080");
    private static final String GEOFEED_LOOKUP_PATH = "/api/v1/geofeed/lookup";
    private static final String ROA_VALIDATE_PATH = "/api/v1/rpki/roa/validate";

    // Async I/O config
    private static final int ASYNC_TIMEOUT_MS = 4000;
    private static final int ASYNC_CAPACITY = 1000;
    private static final int ASYNC_PARALLELISM = 4;

    public static void main(String[] args) throws Exception {
        final StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();

        // Exactly-once checkpointing
        env.enableCheckpointing(30000, CheckpointingMode.EXACTLY_ONCE);
        env.getCheckpointConfig().setCheckpointTimeout(60000);
        env.getCheckpointConfig().setMinPauseBetweenCheckpoints(10000);
        env.getCheckpointConfig().setTolerableCheckpointFailureNumber(3);
        env.getCheckpointConfig().setExternalizedCheckpointCleanup(
            org.apache.flink.streaming.api.environment.CheckpointConfig.ExternalizedCheckpointCleanup.RETAIN_ON_CANCELLATION
        );

        // Watermark strategy for event-time processing
        WatermarkStrategy<L4Response> watermarkStrategy = WatermarkStrategy
            .<L4Response>forBoundedOutOfOrderness(Duration.ofSeconds(5))
            .withTimestampAssigner((event, timestamp) -> parseTimestamp(event.getReceivedAt()))
            .withIdleness(Duration.ofSeconds(30));

        // Kafka source with protobuf deserialization
        KafkaSource<L4Response> kafkaSource = KafkaSource.<L4Response>builder()
            .setBootstrapServers(KAFKA_BROKER)
            .setTopics(INPUT_TOPIC)
            .setGroupId("aegis-flink-geo-enrichment")
            .setStartingOffsets(OffsetsInitializer.latest())
            .setValueOnlyDeserializer(new L4ResponseDeserializationSchema())
            .build();

        // Create DataStream from Kafka
        DataStream<L4Response> rawScans = env.fromSource(
            kafkaSource,
            watermarkStrategy,
            "Kafka Raw L4 Scans"
        );

        // Filter to only OPEN results (active hosts)
        DataStream<L4Response> openHosts = rawScans
            .filter(event -> event.getResult() == L4Response.L4Result.L4_RESULT_OPEN)
            .name("Filter Open Hosts");

        // Async enrichment with Tier 1 geolocation
        DataStream<EnrichedHost> enriched = openHosts
            .asyncIO(
                new Tier1GeolocationEnricher(),
                ASYNC_TIMEOUT_MS,
                ASYNC_CAPACITY
            )
            .name("Tier 1 Geolocation Enrichment")
            .uid("tier1-geo-enrichment")
            .setParallelism(ASYNC_PARALLELISM);

        // Filter only successfully enriched (Tier 1 confidence > 0.95)
        DataStream<EnrichedHost> tier1Enriched = enriched
            .filter(event -> event.getGeoTierUsed() == aegis.types.GeoTier.GEO_TIER_CRYPTOGRAPHIC
                && event.getGeoConfidence() >= 0.95f)
            .name("Filter Tier 1 Results");

        // Kafka sink for enrichment-events topic
        KafkaSink<EnrichedHost> kafkaSink = KafkaSink.<EnrichedHost>builder()
            .setBootstrapServers(KAFKA_BROKER)
            .setRecordSerializer(KafkaRecordSerializationSchema.builder()
                .setTopic(OUTPUT_TOPIC)
                .setValueSerializationSchema(new EnrichedHostSerializationSchema())
                .build())
            .setDeliveryGuarantee(org.apache.flink.connector.kafka.sink.DeliveryGuarantee.EXACTLY_ONCE)
            .setTransactionalIdPrefix("aegis-geo-enrichment")
            .build();

        tier1Enriched.sinkTo(kafkaSink)
            .name("Kafka Enrichment Events Sink")
            .uid("enrichment-events-sink");

        env.execute("AEGIS Tier 1 Geolocation Enrichment Job");
    }

    /**
     * Async function for Tier 1 geolocation enrichment.
     * Performs HTTP calls to enrichment service for geofeed lookup + ROA validation.
     */
    public static class Tier1GeolocationEnricher
            implements AsyncFunction<L4Response, EnrichedHost> {

        private final HttpClient httpClient;
        private final ExecutorService executor;

        public Tier1GeolocationEnricher() {
            this.httpClient = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(2))
                .build();
            this.executor = Executors.newFixedThreadPool(ASYNC_PARALLELISM * 2);
        }

        @Override
        public void asyncInvoke(L4Response input, ResultFuture<EnrichedHost> resultFuture) {
            CompletableFuture.supplyAsync(() -> enrich(input), executor)
                .thenAccept(resultFuture::complete)
                .exceptionally(ex -> {
                    LOG.warn("Tier 1 enrichment failed for IP: {}", ex.getMessage());
                    // Emit unenriched host as fallback (Tier 2 will handle)
                    resultFuture.complete(Collections.singletonList(createFallbackEnriched(input)));
                    return null;
                });
        }

        private EnrichedHost enrich(L4Response input) {
            try {
                String ip = ipToString(input.getTargetIp().toByteArray());

                // Step 1: Geofeed lookup
                GeofeedEntry geofeed = lookupGeofeed(ip);
                if (geofeed == null) {
                    return createFallbackEnriched(input);
                }

                // Step 2: ROA validation
                if (!validateRoa(geofeed.prefix, geofeed.originAs)) {
                    LOG.debug("ROA validation failed for {} AS{}", geofeed.prefix, geofeed.originAs);
                    return createFallbackEnriched(input);
                }

                // Step 3: Build EnrichedHost with Tier 1 data
                return EnrichedHost.newBuilder()
                    .setIpAddress(ip)
                    .setGeoTierUsed(aegis.types.GeoTier.GEO_TIER_CRYPTOGRAPHIC)
                    .setGeoLatitude(geofeed.latitude)
                    .setGeoLongitude(geofeed.longitude)
                    .setGeoCity(geofeed.city != null ? geofeed.city : "")
                    .setGeoCountry(geofeed.country != null ? geofeed.country : "")
                    .setGeoConfidence(0.98f) // High confidence for Tier 1
                    .setPort(input.getTargetPort())
                    .setTransport(input.getTransport())
                    .setReclassifiedProtocol("") // Will be filled by L7 enrichment
                    .setDetectionMethod(aegis.types.DetectionMethod.DETECTION_METHOD_PORT_NUMBER)
                    .setSanitizedBanner("")
                    .setBannerHash("")
                    .setFirstSeen(parseTimestamp(input.getReceivedAt()))
                    .setLastSeen(parseTimestamp(input.getReceivedAt()))
                    .setScanCount(1)
                    .setStatus(aegis.types.HostStatus.HOST_STATUS_ACTIVE)
                    .setSourcePop(input.getSourcePop())
                    .setEventId(input.getEventId())
                    .build();

            } catch (Exception e) {
                LOG.debug("Enrichment error for {}: {}", input.getTargetIp(), e.getMessage());
                return createFallbackEnriched(input);
            }
        }

        private GeofeedEntry lookupGeofeed(String ip) {
            try {
                String url = GEOFEED_API_BASE + GEOFEED_LOOKUP_PATH + "?ip=" + ip;
                HttpRequest request = HttpRequest.newBuilder()
                    .uri(URI.create(url))
                    .timeout(Duration.ofMillis(1500))
                    .GET()
                    .build();

                HttpResponse<String> response = httpClient.send(request, HttpResponse.BodyHandlers.ofString());
                if (response.statusCode() == 200) {
                    return parseGeofeedResponse(response.body());
                }
            } catch (Exception e) {
                LOG.debug("Geofeed lookup failed for {}: {}", ip, e.getMessage());
            }
            return null;
        }

        private boolean validateRoa(String prefix, int originAs) {
            try {
                String url = GEOFEED_API_BASE + ROA_VALIDATE_PATH + "?prefix=" + prefix + "&asn=" + originAs;
                HttpRequest request = HttpRequest.newBuilder()
                    .uri(URI.create(url))
                    .timeout(Duration.ofMillis(1000))
                    .GET()
                    .build();

                HttpResponse<String> response = httpClient.send(request, HttpResponse.BodyHandlers.ofString());
                return response.statusCode() == 200 && response.body().contains("\"valid\":true");
            } catch (Exception e) {
                LOG.debug("ROA validation failed for {}: {}", prefix, e.getMessage());
                return false;
            }
        }

        private EnrichedHost createFallbackEnriched(L4Response input) {
            String ip = ipToString(input.getTargetIp().toByteArray());
            return EnrichedHost.newBuilder()
                .setIpAddress(ip)
                .setGeoTierUsed(aegis.types.GeoTier.GEO_TIER_RTT) // Fallback to Tier 2
                .setPort(input.getTargetPort())
                .setTransport(input.getTransport())
                .setDetectionMethod(aegis.types.DetectionMethod.DETECTION_METHOD_PORT_NUMBER)
                .setStatus(aegis.types.HostStatus.HOST_STATUS_ACTIVE)
                .setSourcePop(input.getSourcePop())
                .setEventId(input.getEventId())
                .setFirstSeen(parseTimestamp(input.getReceivedAt()))
                .setLastSeen(parseTimestamp(input.getReceivedAt()))
                .setScanCount(1)
                .build();
        }

        private String ipToString(byte[] bytes) {
            if (bytes.length == 4) {
                return String.format("%d.%d.%d.%d",
                    bytes[0] & 0xFF, bytes[1] & 0xFF, bytes[2] & 0xFF, bytes[3] & 0xFF);
            } else if (bytes.length == 16) {
                StringBuilder sb = new StringBuilder();
                for (int i = 0; i < 16; i += 2) {
                    if (i > 0) sb.append(':');
                    sb.append(String.format("%02x%02x", bytes[i] & 0xFF, bytes[i + 1] & 0xFF));
                }
                return sb.toString();
            } else {
                StringBuilder sb = new StringBuilder();
                for (int i = 0; i < bytes.length; i++) {
                    if (i > 0) sb.append(':');
                    sb.append(String.format("%02x", bytes[i] & 0xFF));
                }
                return sb.toString();
            }
        }

        private long parseTimestamp(String isoString) {
            try {
                return Instant.parse(isoString).toEpochMilli();
            } catch (Exception e) {
                throw new IllegalArgumentException("Invalid timestamp: " + isoString, e);
            }
        }

        private GeofeedEntry parseGeofeedResponse(String json) {
            // Simple JSON parsing - in production use Jackson/Gson
            // Expected: {"prefix":"192.0.2.0/24","lat":37.7749,"lon":-122.4194,"city":"San Francisco","country":"US","origin_as":64496}
            try {
                String prefix = extractJson(json, "prefix");
                double lat = Double.parseDouble(extractJson(json, "lat"));
                double lon = Double.parseDouble(extractJson(json, "lon"));
                String city = extractJson(json, "city");
                String country = extractJson(json, "country");
                int originAs = Integer.parseInt(extractJson(json, "origin_as"));

                return new GeofeedEntry(prefix, lat, lon, city, country, originAs);
            } catch (Exception e) {
                return null;
            }
        }

        private String extractJson(String json, String key) {
            String searchKey = "\"" + key + "\"";
            int keyIdx = json.indexOf(searchKey);
            if (keyIdx < 0) return "";
            int colonIdx = json.indexOf(':', keyIdx + searchKey.length());
            if (colonIdx < 0) return "";

            int i = colonIdx + 1;
            while (i < json.length() && json.charAt(i) == ' ') i++;

            if (i >= json.length()) return "";

            char c = json.charAt(i);
            if (c == '"') {
                StringBuilder sb = new StringBuilder();
                i++;
                while (i < json.length()) {
                    char ch = json.charAt(i);
                    if (ch == '\\' && i + 1 < json.length()) {
                        sb.append(json.charAt(i + 1));
                        i += 2;
                    } else if (ch == '"') {
                        break;
                    } else {
                        sb.append(ch);
                        i++;
                    }
                }
                return sb.toString();
            } else {
                int start = i;
                while (i < json.length() && json.charAt(i) != ',' && json.charAt(i) != '}' && json.charAt(i) != ' ') {
                    i++;
                }
                return json.substring(start, i).trim();
            }
        }

        @Override
        public void timeout(L4Response input, ResultFuture<EnrichedHost> resultFuture) {
            LOG.warn("Tier 1 enrichment timeout for IP: {}", ipToString(input.getTargetIp().toByteArray()));
            resultFuture.complete(Collections.singletonList(createFallbackEnriched(input)));
        }
    }

    /**
     * Protobuf deserialization schema for L4Response
     */
    public static class L4ResponseDeserializationSchema implements DeserializationSchema<L4Response> {
        @Override
        public L4Response deserialize(byte[] message) {
            try {
                return L4Response.parseFrom(message);
            } catch (Exception e) {
                throw new RuntimeException("Failed to parse L4Response", e);
            }
        }

        @Override
        public boolean isEndOfStream(L4Response nextElement) {
            return false;
        }

        @Override
        public TypeInformation<L4Response> getProducedType() {
            return TypeInformation.of(L4Response.class);
        }
    }

    /**
     * Serialization schema for EnrichedHost to Kafka
     */
    public static class EnrichedHostSerializationSchema
            implements KafkaRecordSerializationSchema<EnrichedHost> {

        @Override
        public byte[] serialize(EnrichedHost element, KafkaSinkContext context, Long timestamp) {
            return element.toByteArray();
        }
    }

    // Geofeed entry DTO
    private static class GeofeedEntry {
        String prefix;
        double latitude;
        double longitude;
        String city;
        String country;
        int originAs;

        GeofeedEntry(String prefix, double lat, double lon, String city, String country, int originAs) {
            this.prefix = prefix;
            this.latitude = lat;
            this.longitude = lon;
            this.city = city;
            this.country = country;
            this.originAs = originAs;
        }
    }
}