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

const DEFINITION: &str = include_str!("../tests/templates.xml");

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

fn definitions(c: &mut Criterion) {
    let mut g = c.benchmark_group("definitions");

    let total_bytes = DEFINITIONS_RAW.iter().fold(0u64, |a, e| a + e.len() as u64);
    g.throughput(Throughput::Bytes(total_bytes));

    //
    // Bench decode
    //
    let mut d = Decoder::new_from_xml(DEFINITION).unwrap();
    g.bench_function("decode/noop", |b| {
        b.iter(|| {
            d.reset();
            for raw in DEFINITIONS_RAW {
                let mut msg = NoopFactory;
                d.decode_slice(black_box(raw), &mut msg).unwrap();
                black_box(msg);
            }
        });
    });
    g.bench_function("decode/text", |b| {
        b.iter(|| {
            d.reset();
            for raw in DEFINITIONS_RAW {
                let mut msg = TextMessageFactory::new();
                d.decode_slice(black_box(raw), &mut msg).unwrap();
                black_box(msg);
            }
        });
    });
    g.bench_function("decode/serde", |b| {
        b.iter(|| {
            d.reset();
            for raw in DEFINITIONS_RAW {
                let msg: cqg_serde::Message = fastlib::from_slice(&mut d, black_box(raw)).unwrap();
                black_box(msg);
            }
        });
    });

    //
    // Bench encode
    //
    let mut e = Encoder::new_from_xml(DEFINITION).unwrap();
    let mut msgs: Vec<TextMessageVisitor> = {
        DEFINITIONS_TEXT
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

    let msgs: Vec<cqg_serde::Message> = {
        let mut d = Decoder::new_from_xml(DEFINITION).unwrap();
        DEFINITIONS_RAW
            .iter()
            .map(|r| fastlib::from_slice(&mut d, r).unwrap())
            .collect()
    };
    g.bench_function("encode/serde", |b| {
        b.iter(|| {
            e.reset();
            for msg in &msgs {
                let raw = fastlib::to_vec(&mut e, black_box(msg)).unwrap();
                black_box(raw);
            }
        });
    });
}

criterion_group!(benches, definitions);
criterion_main!(benches);
