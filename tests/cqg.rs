//! # Tests based on CQG's template
//!
//! See: https://help.cqg.com/apihelp/#!Documents/quotesdirectfixfast.htm
//!
use fastlib::Decoder;
use fastlib::{TextMessageFactory, JsonMessageFactory};

struct TestCaseSeq {
    name: &'static str,
    inputs: Vec<Vec<u8>>,
    results: Vec<&'static str>,
}

fn do_tests_seq(test_cases: Vec<TestCaseSeq>) {
    for tt in test_cases {
        let mut d = Decoder::new_from_xml(include_str!("templates.xml")).unwrap();
        for (i, (input, result)) in tt.inputs.iter().zip(tt.results).enumerate() {
            let mut msg = TextMessageFactory::new();
            d.decode_vec(input.clone(), &mut msg).unwrap();
            assert_eq!(&msg.text, result, "{} failed #{}", tt.name, i + 1);
        }
    }
}

#[test]
fn test_heartbeats() {
    do_tests_seq(vec![
        TestCaseSeq {
            name: "MDHeartbeat",
            inputs: vec![
                vec![0xc0, 0x84, 0x81, 0x23, 0x7a, 0x17, 0x15, 0x15, 0x2c, 0x58, 0x80],
                vec![0x80, 0x82, 0x23, 0x7a, 0x17, 0x15, 0x15, 0x2d, 0x26, 0x90],
                vec![0x80, 0x83, 0x23, 0x7a, 0x17, 0x15, 0x15, 0x2d, 0x74, 0xa0],
            ],
            results: vec![
                "MDHeartbeat=<MessageType=0|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=1|SendingTime=20240606000000000>",
                "MDHeartbeat=<MessageType=0|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=2|SendingTime=20240606000010000>",
                "MDHeartbeat=<MessageType=0|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=3|SendingTime=20240606000020000>",
            ],
        }
    ])
}

#[test]
fn test_logon() {
    do_tests_seq(vec![
        TestCaseSeq {
            name: "MDLogon",
            inputs: vec![
                vec![0xc0, 0x85, 0x81, 0x23, 0x7a, 0x17, 0x15, 0x7a, 0x4d, 0x51, 0x9d, 0x8a],
            ],
            results: vec![
                "MDLogon=<MessageType=A|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=1|SendingTime=20240606212352157|EncryptMethod=0|HeartbeatInt=10>",
            ],
        }
    ])
}

#[test]
fn test_definitions() {
    do_tests_seq(vec![
        TestCaseSeq {
            name: "MDSecurityDefinition",
            inputs: vec![
                vec![0x7F, 0x57, 0xC0, 0x82, 0x07, 0xC4, 0x23, 0x7A, 0x17, 0x15, 0x7A, 0x4D, 0x59, 0x83, 0x07, 0xC6, 0x82, 0x80, 0x09, 0x53, 0x35, 0xE9, 0x00, 0x68, 0x73, 0x5E, 0x80, 0x4D, 0x42, 0x54, 0x53, 0x31, 0xB3, 0x4D, 0x42, 0x54, 0x53, 0x31, 0x33, 0x43, 0x31, 0x30, 0xB0, 0x4D, 0x69, 0x63, 0x72, 0x6F, 0x20, 0x42, 0x69, 0x74, 0x63, 0x6F, 0x69, 0x6E, 0x20, 0x52, 0x65, 0x76, 0x65, 0x72, 0x73, 0x65, 0x20, 0x43, 0x61, 0x6C, 0x20, 0x53, 0x70, 0x72, 0x65, 0x61, 0xE4, 0x4D, 0x42, 0x54, 0x53, 0x31, 0x33, 0x58, 0x32, 0xB4, 0x1C, 0x79, 0x58, 0xFE, 0x46, 0x58, 0x58, 0x58, 0x58, 0xD8, 0x47, 0x4C, 0x42, 0xD8, 0x46, 0x2E, 0x55, 0x53, 0x2E, 0x4D, 0x42, 0x54, 0x57, 0x31, 0x33, 0x58, 0x32, 0xB4, 0x81, 0x80, 0x55, 0x53, 0xC4, 0x83, 0x80, 0x80, 0xC0, 0x43, 0x51, 0x47, 0xC9, 0x81, 0x82, 0xC0, 0x07, 0xEB, 0x31, 0x30, 0xB0, 0x0C, 0x2D, 0xAC, 0x81, 0x81, 0xFF, 0x81, 0x81, 0x81, 0xB4, 0x84, 0x81, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x35, 0x2E, 0xB4, 0x55, 0xFC, 0x82, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x36, 0x2E, 0xB4, 0x5D, 0xE4, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x90, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x91, 0x86, 0x09, 0x53, 0x31, 0x93, 0x23, 0x7A, 0x14, 0x7A, 0x6E, 0x50, 0x46, 0x80, 0x23, 0x7A, 0x14, 0x7A, 0x6A, 0x49, 0x5F, 0xE0, 0x23, 0x7A, 0x14, 0x7E, 0x46, 0x59, 0x2D, 0x80, 0x23, 0x7A, 0x14, 0x7E, 0x46, 0x59, 0x2D, 0x80, 0x00, 0xC8, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x80, 0x80, 0x80],
                vec![0x18, 0xC0, 0x07, 0xC5, 0x23, 0x7A, 0x17, 0x15, 0x7A, 0x4D, 0x59, 0x83, 0x82, 0x80, 0x7F, 0x98, 0x7B, 0x1D, 0x53, 0x80, 0x4D, 0x42, 0x54, 0x53, 0xB1, 0x4D, 0x42, 0x54, 0x53, 0x31, 0x43, 0x31, 0x30, 0xB0, 0x4D, 0x42, 0x54, 0x53, 0x31, 0x56, 0x32, 0xB4, 0x1C, 0x79, 0x58, 0xC1, 0x46, 0x2E, 0x55, 0x53, 0x2E, 0x4D, 0x42, 0x54, 0x57, 0x31, 0x56, 0x32, 0xB4, 0x81, 0x80, 0x83, 0x80, 0x80, 0xC0, 0x43, 0x51, 0x47, 0xC9, 0x81, 0x82, 0x80, 0x80, 0xFF, 0x84, 0x81, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x35, 0x2E, 0xB4, 0x55, 0xFC, 0x82, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x36, 0x2E, 0xB4, 0x5D, 0xE4, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x90, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x91, 0x86, 0x7F, 0xB4, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x00, 0xC8, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x80, 0x80, 0x80],
                vec![0x00, 0xC0, 0x07, 0xC6, 0x23, 0x7A, 0x17, 0x15, 0x7A, 0x4D, 0x59, 0x83, 0x82, 0x80, 0x00, 0xE8, 0x04, 0x62, 0x2D, 0x80, 0x4D, 0x42, 0x54, 0x53, 0x31, 0x58, 0x32, 0xB4, 0x1C, 0x79, 0x58, 0xC0, 0x46, 0x2E, 0x55, 0x53, 0x2E, 0x4D, 0x42, 0x54, 0x57, 0x31, 0x58, 0x32, 0xB4, 0x81, 0x80, 0x83, 0x80, 0x80, 0xC0, 0x43, 0x51, 0x47, 0xC9, 0x81, 0x82, 0x80, 0x80, 0x82, 0x84, 0x81, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x35, 0x2E, 0xB4, 0x55, 0xFC, 0x82, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x36, 0x2E, 0xB4, 0x5D, 0xE4, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x90, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x91, 0x86, 0x7F, 0xB4, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x7D, 0x64, 0x70, 0x30, 0x10, 0x80, 0x00, 0xC8, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x80, 0x80, 0x80],
            ],
            results: vec![
                "MDSecurityDefinition=<MessageType=d|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=964|SendingTime=20240606212353155|TotNumReports=966|Events=<EventType=7|EventDate=20241129|EventTime=220000000>|SecurityGroup=MBTS13|Symbol=MBTS13C100|SecurityName=Micro Bitcoin Reverse Cal Spread|SecurityDesc=MBTS13X24|SecurityID=60714110|SecurityIDSource=100|CFICode=FXXXXX|SecurityExchange=GLBX|CQGSecurityName=F.US.MBTW13X24|StrikePrice=0|Currency=USD|MDFeedTypes=<MDFeedType=CQGC|MarketDepth=0><MDFeedType=CQGI|MarketDepth=1>|InstrAttrib=<InstrAttribType=1003|InstrAttribValue=100>|MaturityMonthYear=202411|MinPriceIncrement=1|MinPriceIncrementAmount=0.1|DisplayFactor=1|ApplID=4|Connections=<ConnectionType=1|ConnectionIPAddress=239.246.5.4|ConnectionPortNumber=11004><ConnectionType=2|ConnectionIPAddress=239.246.6.4|ConnectionPortNumber=12004><ConnectionType=3|ConnectionIPAddress=10.1.0.120|ConnectionPortNumber=10000><ConnectionType=3|ConnectionIPAddress=10.1.0.120|ConnectionPortNumber=10001>|TradingSessions=<TradeDate=20240531|TradSesStartTime=20240530220000000|TradSesOpenTime=20240530211500000|TradSesCloseTime=20240531210000000|TradSesEndTime=20240531210000000><TradeDate=20240603|TradSesStartTime=20240602220000000|TradSesOpenTime=20240602211500000|TradSesCloseTime=20240603210000000|TradSesEndTime=20240603210000000><TradeDate=20240604|TradSesStartTime=20240603220000000|TradSesOpenTime=20240603211500000|TradSesCloseTime=20240604210000000|TradSesEndTime=20240604210000000><TradeDate=20240605|TradSesStartTime=20240604220000000|TradSesOpenTime=20240604211500000|TradSesCloseTime=20240605210000000|TradSesEndTime=20240605210000000><TradeDate=20240606|TradSesStartTime=20240605220000000|TradSesOpenTime=20240605211500000|TradSesCloseTime=20240606210000000|TradSesEndTime=20240606210000000><TradeDate=20240607|TradSesStartTime=20240606220000000|TradSesOpenTime=20240606211500000|TradSesCloseTime=20240607210000000|TradSesEndTime=20240607210000000>>",
                "MDSecurityDefinition=<MessageType=d|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=965|SendingTime=20240606212353155|TotNumReports=966|Events=<EventType=7|EventDate=20241025|EventTime=210000000>|SecurityGroup=MBTS1|Symbol=MBTS1C100|SecurityName=Micro Bitcoin Reverse Cal Spread|SecurityDesc=MBTS1V24|SecurityID=60714049|SecurityIDSource=100|CFICode=FXXXXX|SecurityExchange=GLBX|CQGSecurityName=F.US.MBTW1V24|StrikePrice=0|Currency=USD|MDFeedTypes=<MDFeedType=CQGC|MarketDepth=0><MDFeedType=CQGI|MarketDepth=1>|InstrAttrib=<InstrAttribType=1003|InstrAttribValue=100>|MaturityMonthYear=202410|MinPriceIncrement=1|MinPriceIncrementAmount=0.1|DisplayFactor=1|ApplID=4|Connections=<ConnectionType=1|ConnectionIPAddress=239.246.5.4|ConnectionPortNumber=11004><ConnectionType=2|ConnectionIPAddress=239.246.6.4|ConnectionPortNumber=12004><ConnectionType=3|ConnectionIPAddress=10.1.0.120|ConnectionPortNumber=10000><ConnectionType=3|ConnectionIPAddress=10.1.0.120|ConnectionPortNumber=10001>|TradingSessions=<TradeDate=20240531|TradSesStartTime=20240530220000000|TradSesOpenTime=20240530211500000|TradSesCloseTime=20240531210000000|TradSesEndTime=20240531210000000><TradeDate=20240603|TradSesStartTime=20240602220000000|TradSesOpenTime=20240602211500000|TradSesCloseTime=20240603210000000|TradSesEndTime=20240603210000000><TradeDate=20240604|TradSesStartTime=20240603220000000|TradSesOpenTime=20240603211500000|TradSesCloseTime=20240604210000000|TradSesEndTime=20240604210000000><TradeDate=20240605|TradSesStartTime=20240604220000000|TradSesOpenTime=20240604211500000|TradSesCloseTime=20240605210000000|TradSesEndTime=20240605210000000><TradeDate=20240606|TradSesStartTime=20240605220000000|TradSesOpenTime=20240605211500000|TradSesCloseTime=20240606210000000|TradSesEndTime=20240606210000000><TradeDate=20240607|TradSesStartTime=20240606220000000|TradSesOpenTime=20240606211500000|TradSesCloseTime=20240607210000000|TradSesEndTime=20240607210000000>>",
                "MDSecurityDefinition=<MessageType=d|ApplVerID=8|SenderCompID=CQG|MsgSeqNum=966|SendingTime=20240606212353155|TotNumReports=966|Events=<EventType=7|EventDate=20241129|EventTime=220000000>|SecurityGroup=MBTS1|Symbol=MBTS1C100|SecurityName=Micro Bitcoin Reverse Cal Spread|SecurityDesc=MBTS1X24|SecurityID=60714048|SecurityIDSource=100|CFICode=FXXXXX|SecurityExchange=GLBX|CQGSecurityName=F.US.MBTW1X24|StrikePrice=0|Currency=USD|MDFeedTypes=<MDFeedType=CQGC|MarketDepth=0><MDFeedType=CQGI|MarketDepth=1>|InstrAttrib=<InstrAttribType=1003|InstrAttribValue=100>|MaturityMonthYear=202411|MinPriceIncrement=1|MinPriceIncrementAmount=0.1|DisplayFactor=1|ApplID=4|Connections=<ConnectionType=1|ConnectionIPAddress=239.246.5.4|ConnectionPortNumber=11004><ConnectionType=2|ConnectionIPAddress=239.246.6.4|ConnectionPortNumber=12004><ConnectionType=3|ConnectionIPAddress=10.1.0.120|ConnectionPortNumber=10000><ConnectionType=3|ConnectionIPAddress=10.1.0.120|ConnectionPortNumber=10001>|TradingSessions=<TradeDate=20240531|TradSesStartTime=20240530220000000|TradSesOpenTime=20240530211500000|TradSesCloseTime=20240531210000000|TradSesEndTime=20240531210000000><TradeDate=20240603|TradSesStartTime=20240602220000000|TradSesOpenTime=20240602211500000|TradSesCloseTime=20240603210000000|TradSesEndTime=20240603210000000><TradeDate=20240604|TradSesStartTime=20240603220000000|TradSesOpenTime=20240603211500000|TradSesCloseTime=20240604210000000|TradSesEndTime=20240604210000000><TradeDate=20240605|TradSesStartTime=20240604220000000|TradSesOpenTime=20240604211500000|TradSesCloseTime=20240605210000000|TradSesEndTime=20240605210000000><TradeDate=20240606|TradSesStartTime=20240605220000000|TradSesOpenTime=20240605211500000|TradSesCloseTime=20240606210000000|TradSesEndTime=20240606210000000><TradeDate=20240607|TradSesStartTime=20240606220000000|TradSesOpenTime=20240606211500000|TradSesCloseTime=20240607210000000|TradSesEndTime=20240607210000000>>",
            ],
        }
    ])
}

fn do_tests_seq_json(test_cases: Vec<TestCaseSeq>) {
    for tt in test_cases {
        let mut d = Decoder::new_from_xml(include_str!("templates.xml")).unwrap();
        for (i, (input, result)) in tt.inputs.iter().zip(tt.results).enumerate() {
            let mut msg = JsonMessageFactory::new();
            d.decode_vec(input.clone(), &mut msg).unwrap();
            assert_eq!(&msg.text, result, "{} failed #{}", tt.name, i + 1);
        }
    }
}

#[test]
fn test_heartbeats_json() {
    do_tests_seq_json(vec![
        TestCaseSeq {
            name: "MDHeartbeat/MDLogon/MDSecurityDefinition",
            inputs: vec![
                vec![0xc0, 0x84, 0x81, 0x23, 0x7a, 0x17, 0x15, 0x15, 0x2c, 0x58, 0x80],
                vec![0xc0, 0x85, 0x81, 0x23, 0x7a, 0x17, 0x15, 0x7a, 0x4d, 0x51, 0x9d, 0x8a],
                vec![0x7F, 0x57, 0xC0, 0x82, 0x07, 0xC4, 0x23, 0x7A, 0x17, 0x15, 0x7A, 0x4D, 0x59, 0x83, 0x07, 0xC6, 0x82, 0x80, 0x09, 0x53, 0x35, 0xE9, 0x00, 0x68, 0x73, 0x5E, 0x80, 0x4D, 0x42, 0x54, 0x53, 0x31, 0xB3, 0x4D, 0x42, 0x54, 0x53, 0x31, 0x33, 0x43, 0x31, 0x30, 0xB0, 0x4D, 0x69, 0x63, 0x72, 0x6F, 0x20, 0x42, 0x69, 0x74, 0x63, 0x6F, 0x69, 0x6E, 0x20, 0x52, 0x65, 0x76, 0x65, 0x72, 0x73, 0x65, 0x20, 0x43, 0x61, 0x6C, 0x20, 0x53, 0x70, 0x72, 0x65, 0x61, 0xE4, 0x4D, 0x42, 0x54, 0x53, 0x31, 0x33, 0x58, 0x32, 0xB4, 0x1C, 0x79, 0x58, 0xFE, 0x46, 0x58, 0x58, 0x58, 0x58, 0xD8, 0x47, 0x4C, 0x42, 0xD8, 0x46, 0x2E, 0x55, 0x53, 0x2E, 0x4D, 0x42, 0x54, 0x57, 0x31, 0x33, 0x58, 0x32, 0xB4, 0x81, 0x80, 0x55, 0x53, 0xC4, 0x83, 0x80, 0x80, 0xC0, 0x43, 0x51, 0x47, 0xC9, 0x81, 0x82, 0xC0, 0x07, 0xEB, 0x31, 0x30, 0xB0, 0x0C, 0x2D, 0xAC, 0x81, 0x81, 0xFF, 0x81, 0x81, 0x81, 0xB4, 0x84, 0x81, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x35, 0x2E, 0xB4, 0x55, 0xFC, 0x82, 0x32, 0x33, 0x39, 0x2E, 0x32, 0x34, 0x36, 0x2E, 0x36, 0x2E, 0xB4, 0x5D, 0xE4, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x90, 0x83, 0x31, 0x30, 0x2E, 0x31, 0x2E, 0x30, 0x2E, 0x31, 0x32, 0xB0, 0x4E, 0x91, 0x86, 0x09, 0x53, 0x31, 0x93, 0x23, 0x7A, 0x14, 0x7A, 0x6E, 0x50, 0x46, 0x80, 0x23, 0x7A, 0x14, 0x7A, 0x6A, 0x49, 0x5F, 0xE0, 0x23, 0x7A, 0x14, 0x7E, 0x46, 0x59, 0x2D, 0x80, 0x23, 0x7A, 0x14, 0x7E, 0x46, 0x59, 0x2D, 0x80, 0x00, 0xC8, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x02, 0x0C, 0x1C, 0x23, 0x20, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x81, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x03, 0x5C, 0x6B, 0x14, 0x80, 0x80, 0x80, 0x80],
            ],
            results: vec![
                r#"{"MDHeartbeat":{"MessageType":"0","ApplVerID":"8","SenderCompID":"CQG","MsgSeqNum":1,"SendingTime":20240606000000000}}"#,
                r#"{"MDLogon":{"MessageType":"A","ApplVerID":"8","SenderCompID":"CQG","MsgSeqNum":1,"SendingTime":20240606212352157,"EncryptMethod":0,"HeartbeatInt":10}}"#,
                r#"{"MDSecurityDefinition":{"MessageType":"d","ApplVerID":"8","SenderCompID":"CQG","MsgSeqNum":964,"SendingTime":20240606212353155,"TotNumReports":966,"Events":[{"EventType":7,"EventDate":20241129,"EventTime":220000000}],"SecurityGroup":"MBTS13","Symbol":"MBTS13C100","SecurityName":"Micro Bitcoin Reverse Cal Spread","SecurityDesc":"MBTS13X24","SecurityID":60714110,"SecurityIDSource":100,"CFICode":"FXXXXX","SecurityExchange":"GLBX","CQGSecurityName":"F.US.MBTW13X24","StrikePrice":0,"Currency":"USD","MDFeedTypes":[{"MDFeedType":"CQGC","MarketDepth":0},{"MDFeedType":"CQGI","MarketDepth":1}],"InstrAttrib":[{"InstrAttribType":1003,"InstrAttribValue":"100"}],"MaturityMonthYear":202411,"MinPriceIncrement":1,"MinPriceIncrementAmount":0.1,"DisplayFactor":1,"ApplID":"4","Connections":[{"ConnectionType":1,"ConnectionIPAddress":"239.246.5.4","ConnectionPortNumber":11004},{"ConnectionType":2,"ConnectionIPAddress":"239.246.6.4","ConnectionPortNumber":12004},{"ConnectionType":3,"ConnectionIPAddress":"10.1.0.120","ConnectionPortNumber":10000},{"ConnectionType":3,"ConnectionIPAddress":"10.1.0.120","ConnectionPortNumber":10001}],"TradingSessions":[{"TradeDate":20240531,"TradSesStartTime":20240530220000000,"TradSesOpenTime":20240530211500000,"TradSesCloseTime":20240531210000000,"TradSesEndTime":20240531210000000},{"TradeDate":20240603,"TradSesStartTime":20240602220000000,"TradSesOpenTime":20240602211500000,"TradSesCloseTime":20240603210000000,"TradSesEndTime":20240603210000000},{"TradeDate":20240604,"TradSesStartTime":20240603220000000,"TradSesOpenTime":20240603211500000,"TradSesCloseTime":20240604210000000,"TradSesEndTime":20240604210000000},{"TradeDate":20240605,"TradSesStartTime":20240604220000000,"TradSesOpenTime":20240604211500000,"TradSesCloseTime":20240605210000000,"TradSesEndTime":20240605210000000},{"TradeDate":20240606,"TradSesStartTime":20240605220000000,"TradSesOpenTime":20240605211500000,"TradSesCloseTime":20240606210000000,"TradSesEndTime":20240606210000000},{"TradeDate":20240607,"TradSesStartTime":20240606220000000,"TradSesOpenTime":20240606211500000,"TradSesCloseTime":20240607210000000,"TradSesEndTime":20240607210000000}]}}"#
            ],
        }
    ])
}
