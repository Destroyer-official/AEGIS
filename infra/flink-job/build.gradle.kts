plugins {
    java
    id("com.gradleup.shadow") version "8.3.6"
    id("com.google.protobuf") version "0.9.4"
}

group = "com.aegis.flink"
version = "0.1.0"

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

repositories {
    mavenCentral()
    maven("https://repo1.maven.org/maven2/")
}

protobuf {
    protoc {
        artifact = "com.google.protobuf:protoc:3.25.2"
    }
}

dependencies {
    // Flink core
    implementation("org.apache.flink:flink-streaming-java:1.20.0")
    implementation("org.apache.flink:flink-clients:1.20.0")
    implementation("org.apache.flink:flink-runtime-web:1.20.0")
    implementation("org.apache.flink:flink-table-api-java-bridge:1.20.0")
    implementation("org.apache.flink:flink-table-planner-loader:1.20.0")
    implementation("org.apache.flink:flink-table-runtime:1.20.0")
    implementation("org.apache.flink:flink-json:1.20.0")

    // Kafka connector
    implementation("org.apache.flink:flink-connector-base:1.20.0")
    implementation("org.apache.flink:flink-connector-kafka:3.3.0-1.20")

    // Iceberg
    implementation("org.apache.iceberg:iceberg-flink-runtime-1.20:1.10.0")
    implementation("org.apache.iceberg:iceberg-nessie:1.10.0")
    implementation("org.apache.iceberg:iceberg-aws:1.10.0")

    // S3/MinIO support
    implementation("software.amazon.awssdk:s3:2.25.0")
    implementation("software.amazon.awssdk:sts:2.25.0")
    implementation("org.apache.hadoop:hadoop-aws:3.3.6")

    // Protobuf runtime
    implementation("com.google.protobuf:protobuf-java:3.25.2")
    implementation("com.google.protobuf:protobuf-java-util:3.25.2")

    // Logging
    implementation("org.slf4j:slf4j-simple:2.0.13")
}

sourceSets {
    main {
        java {
            srcDir("build/generated/source/proto/main/java")
        }
    }
}

tasks.jar {
    manifest {
        attributes(
            "Main-Class" to "com.aegis.flink.GeolocationEnrichmentJob"
        )
    }
}

tasks.register<Jar>("deduplicationJar") {
    archiveClassifier.set("dedup")
    manifest {
        attributes(
            "Main-Class" to "com.aegis.flink.DeduplicationJob"
        )
    }
    from(tasks.shadowJar.get().outputs)
}

tasks.shadowJar {
    isZip64 = true
    archiveClassifier.set("")
    manifest {
        attributes(
            "Main-Class" to "com.aegis.flink.GeolocationEnrichmentJob"
        )
    }
    mergeServiceFiles()
}

tasks.build {
    dependsOn(tasks.shadowJar)
}

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
}
