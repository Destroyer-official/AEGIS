package com.aegis.flink;

import aegis.types.L4Response;
import org.apache.flink.api.common.eventtime.WatermarkStrategy;
import org.apache.flink.api.common.serialization.DeserializationSchema;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.connector.kafka.source.KafkaSource;
import org.apache.flink.connector.kafka.source.enumerator.initializer.OffsetsInitializer;
import org.apache.flink.streaming.api.CheckpointingMode;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.apache.flink.streaming.api.functions.windowing.ProcessWindowFunction;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.apache.flink.streaming.api.windowing.windows.TimeWindow;
import org.apache.flink.table.api.bridge.java.StreamTableEnvironment;
import org.apache.flink.util.Collector;

import java.time.Duration;
import java.time.Instant;

/**
 * AEGIS Flink Deduplication Job
 *
 * Uses Flink DataStream API with protobuf deserialization to:
 * 1. Read raw scans from Kafka topic raw-l4-scans (protobuf binary)
 * 2. Deduplicate within 1-minute tumbling windows by host_ip + port_number
 * 3. Write deduplicated results to Iceberg aegis.services table via Table API
 *
 * Per S3.3: Flink consumes topics to execute real-time deduplication via
 * tumbling time-window operators before data persistence.
 */
public class DeduplicationJob {

    private static final String KAFKA_BROKER = "aegis-kafka:9092";
    private static final String TOPIC = "raw-l4-scans";
    private static final String CATALOG = "aegis";
    private static final String NAMESPACE = "aegis";
    private static final String TABLE = "services";

    public static void main(String[] args) throws Exception {
        final StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();

        // Exactly-once checkpointing per S3.3
        env.enableCheckpointing(60000, CheckpointingMode.EXACTLY_ONCE);
        env.getCheckpointConfig().setCheckpointTimeout(120000);
        env.getCheckpointConfig().setMinPauseBetweenCheckpoints(30000);
        env.getCheckpointConfig().setTolerableCheckpointFailureNumber(3);

        // Configure watermark strategy for event-time processing
        WatermarkStrategy<L4Response> watermarkStrategy = WatermarkStrategy
            .<L4Response>forBoundedOutOfOrderness(Duration.ofSeconds(10))
            .withTimestampAssigner((event, timestamp) -> parseTimestamp(event.getReceivedAt()));

        // Create Kafka source with protobuf deserialization
        KafkaSource<L4Response> kafkaSource = KafkaSource.<L4Response>builder()
            .setBootstrapServers(KAFKA_BROKER)
            .setTopics(TOPIC)
            .setGroupId("aegis-flink-dedup")
            .setStartingOffsets(OffsetsInitializer.earliest())
            .setValueOnlyDeserializer(new L4ResponseDeserializationSchema())
            .build();

        // Create DataStream from Kafka
        DataStream<L4Response> rawScans = env.fromSource(
            kafkaSource,
            watermarkStrategy,
            "Kafka Raw L4 Scans"
        );

        // Key by host_ip:port_number for deduplication
        DataStream<L4Response> deduped = rawScans
            .keyBy(event -> {
                String ip = ipToString(event.getTargetIp().toByteArray());
                return ip + ":" + event.getTargetPort();
            })
            .window(TumblingEventTimeWindows.of(Time.minutes(1)))
            .process(new DeduplicationWindowFunction());

        // Convert to Table and write to Iceberg
        final StreamTableEnvironment tableEnv = StreamTableEnvironment.create(env);

        // Register Iceberg catalog via Nessie (catalog-impl per Nessie+Flink docs)
        tableEnv.executeSql(
            "CREATE CATALOG IF NOT EXISTS " + CATALOG + " WITH (" +
            "  'type' = 'iceberg'," +
            "  'catalog-impl' = 'org.apache.iceberg.nessie.NessieCatalog'," +
            "  'uri' = 'http://aegis-polaris:19120/api/v1'," +
            "  'ref' = 'main'," +
            "  'warehouse' = 's3://aegis-warehouse/'," +
            "  'io-impl' = 'org.apache.iceberg.aws.s3.S3FileIO'," +
            "  's3.endpoint' = 'http://aegis-minio:9000'," +
            "  's3.path-style-access' = 'true'," +
            "  's3.access-key-id' = '" + System.getenv("MINIO_ACCESS_KEY") + "'," +
            "  's3.secret-access-key' = '" + System.getenv("MINIO_SECRET_KEY") + "'" +
            ")"
        );

        tableEnv.executeSql("USE CATALOG " + CATALOG);
        tableEnv.executeSql("CREATE DATABASE IF NOT EXISTS " + NAMESPACE);
        tableEnv.executeSql("USE " + NAMESPACE);

        // Create Iceberg sink table - matches Trino §7.2 Service spec (22 cols)
        tableEnv.executeSql(
            "CREATE TABLE IF NOT EXISTS " + TABLE + " (" +
            "  host_ip               STRING," +
            "  transport_protocol    STRING," +
            "  port_number           BIGINT," +
            "  reclassified_protocol STRING," +
            "  detection_method      STRING," +
            "  raw_banner            STRING," +
            "  sanitized_banner      STRING," +
            "  banner_hash           STRING," +
            "  ja4_fingerprint       STRING," +
            "  ja4s_fingerprint      STRING," +
            "  tls_version           STRING," +
            "  cipher_suite          STRING," +
            "  certificate_chain     STRING," +
            "  http_method           STRING," +
            "  http_status           BIGINT," +
            "  ssh_version           STRING," +
            "  ssh_kex_algorithms    STRING," +
            "  cves                  ARRAY<STRING>," +
            "  cvss_scores           ARRAY<DOUBLE>," +
            "  first_seen            BIGINT," +
            "  last_seen             BIGINT," +
            "  status                STRING" +
            ") WITH (" +
            "  'format-version' = '2'," +
            "  'write.upsert.enabled' = 'true'" +
            ")"
        );

        // Map L4Response to a flat POJO for Table API
        DataStream<ServiceRecord> records = deduped.map(event -> mapToServiceRecord(event))
            .returns(ServiceRecord.class);

        // Register as temporary view
        tableEnv.createTemporaryView("deduped_scans", records);

        // Insert into Iceberg
        tableEnv.executeSql(
            "INSERT INTO " + TABLE + " " +
            "SELECT " +
            "  host_ip, transport_protocol, port_number," +
            "  reclassified_protocol, detection_method," +
            "  raw_banner, sanitized_banner, banner_hash," +
            "  ja4_fingerprint, ja4s_fingerprint, tls_version, cipher_suite, certificate_chain," +
            "  http_method, http_status, ssh_version, ssh_kex_algorithms," +
            "  cves, cvss_scores, first_seen, last_seen, status " +
            "FROM deduped_scans"
        );

        env.execute("AEGIS Flink Deduplication Job");
    }

    /**
     * Map L4Response protobuf to flat ServiceRecord for Iceberg
     */
    private static ServiceRecord mapToServiceRecord(L4Response event) {
        byte[] ipBytes = event.getTargetIp().toByteArray();
        String ip = ipToString(ipBytes);

        long ts = parseTimestamp(event.getReceivedAt());

        // Map L4Result enum to string
        String resultStr;
        switch (event.getResult()) {
            case L4_RESULT_OPEN: resultStr = "open"; break;
            case L4_RESULT_CLOSED: resultStr = "closed"; break;
            case L4_RESULT_FILTERED: resultStr = "filtered"; break;
            case L4_RESULT_HOST_ALIVE: resultStr = "host_alive"; break;
            default: resultStr = "unknown";
        }

        // Map Transport enum to string
        String transportStr;
        switch (event.getTransport()) {
            case TRANSPORT_TCP: transportStr = "TCP"; break;
            case TRANSPORT_UDP: transportStr = "UDP"; break;
            default: transportStr = "UNKNOWN";
        }

        // Detection method (L4 scan uses port number detection)
        String detectionMethod = "PORT_NUMBER";

        // Status based on result
        String status = (event.getResult() == L4Response.L4Result.L4_RESULT_OPEN) ? "active" : "inactive";

        return new ServiceRecord(
            ip,
            transportStr,
            event.getTargetPort(),
            resultStr,
            detectionMethod,
            null,  // raw_banner
            "",    // sanitized_banner
            "",    // banner_hash
            "",    // ja4_fingerprint
            "",    // ja4s_fingerprint
            "",    // tls_version
            "",    // cipher_suite
            "",    // certificate_chain
            "",    // http_method
            0L,    // http_status
            "",    // ssh_version
            "",    // ssh_kex_algorithms
            new String[0],  // cves
            new Double[0],  // cvss_scores
            ts,
            ts,
            status
        );
    }

    private static long parseTimestamp(String isoString) {
        try {
            return Instant.parse(isoString).toEpochMilli();
        } catch (Exception e) {
            throw new IllegalArgumentException("Invalid timestamp: " + isoString, e);
        }
    }

    private static String ipToString(byte[] bytes) {
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
     * Deduplication window function - keeps the latest event per host:port
     */
    public static class DeduplicationWindowFunction
            extends ProcessWindowFunction<L4Response, L4Response, String, TimeWindow> {

        @Override
        public void process(String key, Context context, Iterable<L4Response> elements, Collector<L4Response> out) {
            L4Response latest = null;
            long latestTimestamp = Long.MIN_VALUE;

            for (L4Response event : elements) {
                long ts = parseTimestamp(event.getReceivedAt());
                if (ts > latestTimestamp) {
                    latestTimestamp = ts;
                    latest = event;
                }
            }

            if (latest != null) {
                out.collect(latest);
            }
        }
    }

    /**
     * Flat POJO for Iceberg services table - all 22 columns
     */
    public static class ServiceRecord {
        public String host_ip;
        public String transport_protocol;
        public long port_number;
        public String reclassified_protocol;
        public String detection_method;
        public String raw_banner;
        public String sanitized_banner;
        public String banner_hash;
        public String ja4_fingerprint;
        public String ja4s_fingerprint;
        public String tls_version;
        public String cipher_suite;
        public String certificate_chain;
        public String http_method;
        public long http_status;
        public String ssh_version;
        public String ssh_kex_algorithms;
        public String[] cves;
        public Double[] cvss_scores;
        public long first_seen;
        public long last_seen;
        public String status;

        public ServiceRecord() {}

        public ServiceRecord(
            String host_ip, String transport_protocol, long port_number,
            String reclassified_protocol, String detection_method,
            String raw_banner, String sanitized_banner, String banner_hash,
            String ja4_fingerprint, String ja4s_fingerprint,
            String tls_version, String cipher_suite, String certificate_chain,
            String http_method, long http_status,
            String ssh_version, String ssh_kex_algorithms,
            String[] cves, Double[] cvss_scores,
            long first_seen, long last_seen, String status
        ) {
            this.host_ip = host_ip;
            this.transport_protocol = transport_protocol;
            this.port_number = port_number;
            this.reclassified_protocol = reclassified_protocol;
            this.detection_method = detection_method;
            this.raw_banner = raw_banner;
            this.sanitized_banner = sanitized_banner;
            this.banner_hash = banner_hash;
            this.ja4_fingerprint = ja4_fingerprint;
            this.ja4s_fingerprint = ja4s_fingerprint;
            this.tls_version = tls_version;
            this.cipher_suite = cipher_suite;
            this.certificate_chain = certificate_chain;
            this.http_method = http_method;
            this.http_status = http_status;
            this.ssh_version = ssh_version;
            this.ssh_kex_algorithms = ssh_kex_algorithms;
            this.cves = cves;
            this.cvss_scores = cvss_scores;
            this.first_seen = first_seen;
            this.last_seen = last_seen;
            this.status = status;
        }
    }
}