/* AEGIS AF_XDP Redirect Program
 * Parses Ethernet/IP/TCP and redirects to XSKMAP for userspace processing.
 * Attach: ip link set dev <ifname> xdp obj xdp_redirect.o sec xdp
 */

#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/ip.h>
#include <linux/ipv6.h>
#include <linux/tcp.h>
#include <linux/udp.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>

#define XDP_PASS 1
#define XDP_REDIRECT 4
#define XDP_DROP 2

struct {
    __uint(type, BPF_MAP_TYPE_XSKMAP);
    __uint(key_size, sizeof(__u32));
    __uint(value_size, sizeof(__u32));
    __uint(max_entries, 64);
} xsks_map SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
    __uint(key_size, sizeof(__u32));
    __uint(value_size, sizeof(__u32));
} events SEC(".maps");

struct event_data {
    __u32 src_ip;
    __u32 dst_ip;
    __u16 src_port;
    __u16 dst_port;
    __u8  protocol;
    __u8  tcp_flags;
};

SEC("xdp")
int xdp_redirect_prog(struct xdp_md *ctx) {
    void *data_end = (void *)(long)ctx->data_end;
    void *data = (void *)(long)ctx->data;

    struct ethhdr *eth = data;
    if ((void *)(eth + 1) > data_end)
        return XDP_PASS;

    __u16 ethertype = bpf_ntohs(eth->h_proto);

    if (ethertype != ETH_P_IP && ethertype != ETH_P_IPV6)
        return XDP_PASS;

    __u32 queue_id = ctx->queue_index;

    struct event_data evt = {};

    if (ethertype == ETH_P_IP) {
        struct iphdr *ip = (void *)(eth + 1);
        if ((void *)(ip + 1) > data_end)
            return XDP_PASS;

        evt.src_ip = ip->saddr;
        evt.dst_ip = ip->daddr;
        evt.protocol = ip->protocol;

        if (ip->protocol == IPPROTO_TCP) {
            struct tcphdr *tcp = (void *)ip + (ip->ihl * 4);
            if ((void *)(tcp + 1) > data_end)
                return XDP_PASS;

            evt.src_port = bpf_ntohs(tcp->source);
            evt.dst_port = bpf_ntohs(tcp->dest);

            __u8 flags = 0;
            if (tcp->syn) flags |= 0x02;
            if (tcp->ack) flags |= 0x10;
            if (tcp->fin) flags |= 0x01;
            if (tcp->rst) flags |= 0x04;
            evt.tcp_flags = flags;
        } else if (ip->protocol == IPPROTO_UDP) {
            struct udphdr *udp = (void *)ip + (ip->ihl * 4);
            if ((void *)(udp + 1) > data_end)
                return XDP_PASS;

            evt.src_port = bpf_ntohs(udp->source);
            evt.dst_port = bpf_ntohs(udp->dest);
        } else {
            return XDP_PASS;
        }
    } else {
        struct ipv6hdr *ip6 = (void *)(eth + 1);
        if ((void *)(ip6 + 1) > data_end)
            return XDP_PASS;

        evt.protocol = ip6->nexthdr;

        if (ip6->nexthdr == IPPROTO_TCP) {
            struct tcphdr *tcp = (void *)(ip6 + 1);
            if ((void *)(tcp + 1) > data_end)
                return XDP_PASS;

            evt.src_port = bpf_ntohs(tcp->source);
            evt.dst_port = bpf_ntohs(tcp->dest);

            __u8 flags = 0;
            if (tcp->syn) flags |= 0x02;
            if (tcp->ack) flags |= 0x10;
            if (tcp->fin) flags |= 0x01;
            if (tcp->rst) flags |= 0x04;
            evt.tcp_flags = flags;
        } else if (ip6->nexthdr == IPPROTO_UDP) {
            struct udphdr *udp = (void *)(ip6 + 1);
            if ((void *)(udp + 1) > data_end)
                return XDP_PASS;

            evt.src_port = bpf_ntohs(udp->source);
            evt.dst_port = bpf_ntohs(udp->dest);
        } else {
            return XDP_PASS;
        }
    }

    bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU, &evt, sizeof(evt));

    return bpf_redirect_map(&xsks_map, queue_id, XDP_PASS);
}

char LICENSE[] SEC("license") = "GPL";
