use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fastlib::{Decoder, Encoder, MessageFactory, TextMessageFactory, TextMessageVisitor, Value};
use std::hint::black_box;

use crate::cqg::{DEFINITIONS_RAW, DEFINITIONS_TEXT};

#[path = "../tests/cqg-serde.rs"]
#[allow(dead_code)]
mod cqg_serde;

#[path = "../tests/cqg.rs"]
#[allow(dead_code)]
mod cqg;

const TEMPLATES_CQG: &str = include_str!("../tests/templates.xml");
const TEMPLATES_BENCH: &str = include_str!("benchmark.xml");

struct NoopFactory;

impl MessageFactory for NoopFactory {
    fn start_template(&mut self, _id: u32, _name: &str) {}
    fn stop_template(&mut self) {}
    fn set_value(&mut self, _id: u32, _name: &str, value: Option<Value>) {
        black_box(value);
    }
    fn start_sequence(&mut self, _id: u32, _name: &str, _length: u32) {}
    fn start_sequence_item(&mut self, _index: u32) {}
    fn stop_sequence_item(&mut self) {}
    fn stop_sequence(&mut self) {}
    fn start_group(&mut self, _name: &str) {}
    fn stop_group(&mut self) {}
    fn start_template_ref(&mut self, _name: &str, _dynamic: bool) {}
    fn stop_template_ref(&mut self) {}
}

fn bench_suite(
    c: &mut Criterion,
    name: &str,
    templates: &str,
    texts: &[String],
    raws: &[Vec<u8>],
    with_serde: bool,
) {
    let mut e = Encoder::new_from_xml(templates).unwrap();
    let mut d = Decoder::new_from_xml(templates).unwrap();
    let mut g = c.benchmark_group(name);

    let total_bytes = raws.iter().fold(0u64, |a, e| a + e.len() as u64);
    g.throughput(Throughput::Bytes(total_bytes));

    g.bench_function("decode/noop", |b| {
        b.iter(|| {
            d.reset();
            for raw in raws {
                let mut msg = NoopFactory;
                d.decode_slice(raw, &mut msg).unwrap();
                black_box(msg);
            }
        });
    });

    let mut msgs: Vec<TextMessageVisitor> = {
        texts
            .iter()
            .map(|t| TextMessageVisitor::from_text(t).unwrap())
            .collect()
    };
    g.bench_function("encode/text", |b| {
        b.iter(|| {
            e.reset();
            for msg in &mut msgs {
                let raw = e.encode_vec(msg).unwrap();
                black_box(raw);
            }
        });
    });
    g.bench_function("decode/text", |b| {
        b.iter(|| {
            d.reset();
            for raw in raws {
                let mut msg = TextMessageFactory::new();
                d.decode_slice(raw, &mut msg).unwrap();
                black_box(msg);
            }
        });
    });

    if with_serde {
        let msgs: Vec<cqg_serde::Message> = {
            let mut d = Decoder::new_from_xml(templates).unwrap();
            raws.iter()
                .map(|r| fastlib::from_slice(&mut d, r).unwrap())
                .collect()
        };
        g.bench_function("encode/serde", |b| {
            b.iter(|| {
                e.reset();
                for msg in &msgs {
                    let raw = fastlib::to_vec(&mut e, msg).unwrap();
                    black_box(raw);
                }
            });
        });
        g.bench_function("decode/serde", |b| {
            b.iter(|| {
                d.reset();
                for raw in raws {
                    let msg: cqg_serde::Message = fastlib::from_slice(&mut d, raw).unwrap();
                    black_box(msg);
                }
            });
        });
    }
}

fn definitions(c: &mut Criterion) {
    let texts: Vec<String> = DEFINITIONS_TEXT.iter().map(ToString::to_string).collect();
    let raws: Vec<Vec<u8>> = DEFINITIONS_RAW.iter().map(|r| r.to_vec()).collect();
    bench_suite(c, "definitions", TEMPLATES_CQG, &texts, &raws, true);
}

fn messages(c: &mut Criterion) {
    let texts: Vec<String> = {
        (0..100u32).map(|n| {
            let text = if n % 3 == 0 {
                String::new()
            } else {
                format!("|text=Message #{n}")
            };
            format!("Message=<MsgSeqNum={}|SendingTime={}|Symbol=F.US.EZY{:06}|SecurityID=EZY{}|SecurityDesc=Description {}{}|Price1={}.25|Price2={}.25|Size={}|Currency=USD|Level={}>",
                n+1,
                20_240_606_212_353_155u64 + u64::from(n) * 7,
                n * 13,
                n / 10,
                n / 4,
                text,
                5000 + n % 17,
                5000 + n % 17,
                100 + (n * 31) % 250,
                1 + n % 2
            )
        }).collect()
    };
    let raws: Vec<Vec<u8>> = {
        let mut e = Encoder::new_from_xml(TEMPLATES_BENCH).unwrap();
        texts
            .iter()
            .map(|t| {
                let mut v = TextMessageVisitor::from_text(t).unwrap();
                e.encode_vec(&mut v).unwrap()
            })
            .collect()
    };
    bench_suite(c, "messages", TEMPLATES_BENCH, &texts, &raws, false);
}

criterion_group!(benches, messages, definitions);
criterion_main!(benches);
