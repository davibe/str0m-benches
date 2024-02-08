use std::io::Cursor;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

use pcap_file::pcap::PcapReader;
use str0m::format::Codec;
use str0m::format::PayloadParams;
use str0m::net::Receive;
use str0m::rtp::ExtensionMap;
use str0m::rtp::RtpHeader;
use str0m::{Event, Input, Output, Rtc, RtcError};
use tracing::Span;

pub struct TestRtc {
    pub span: Span,
    pub rtc: Rtc,
    pub start: Instant,
    pub last: Instant,
    pub events: Option<Vec<(Instant, Event)>>,
    pub media_data_count: usize,
}

impl TestRtc {
    pub fn new(span: Span, capture: bool) -> Self {
        Self::new_with_rtc(span, Rtc::new(), capture)
    }

    pub fn new_with_rtc(span: Span, rtc: Rtc, capture: bool) -> Self {
        let now = Instant::now();
        TestRtc {
            span,
            rtc,
            start: now,
            last: now,
            events: if capture { Some(vec![]) } else { None },
            media_data_count: 0,
        }
    }

    pub fn params_vp8(&self) -> PayloadParams {
        self.rtc
            .codec_config()
            .find(|p| p.spec().codec == Codec::Vp8)
            .cloned()
            .unwrap()
    }

    pub fn params_vp9(&self) -> PayloadParams {
        self.rtc
            .codec_config()
            .find(|p| p.spec().codec == Codec::Vp9)
            .cloned()
            .unwrap()
    }

}

pub fn progress(l: &mut TestRtc, r: &mut TestRtc) -> Result<(), RtcError> {
    let (from, to) = if l.last < r.last { (l, r) } else { (r, l) };

    loop {
        from.span
            .in_scope(|| from.rtc.handle_input(Input::Timeout(from.last)))?;

        match from.span.in_scope(|| from.rtc.poll_output())? {
            Output::Timeout(v) => {
                let tick = from.last + Duration::from_millis(10);
                from.last = if v == from.last { tick } else { tick.min(v) };
                break;
            }
            Output::Transmit(v) => {
                let data = v.contents;
                let input = Input::Receive(
                    from.last,
                    Receive {
                        proto: v.proto,
                        source: v.source,
                        destination: v.destination,
                        contents: (&*data).try_into()?,
                    },
                );
                to.span.in_scope(|| to.rtc.handle_input(input))?;
            }
            Output::Event(v) => {
                match v {
                    Event::MediaData(_) => from.media_data_count += 1,
                    _ => {}
                }

                if let Some(events) = from.events.as_mut() {
                    events.push((from.last, v));
                }
            }
        }
    }

    Ok(())
}

impl Deref for TestRtc {
    type Target = Rtc;

    fn deref(&self) -> &Self::Target {
        &self.rtc
    }
}

impl DerefMut for TestRtc {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rtc
    }
}

pub type PcapData = Vec<(Duration, RtpHeader, Vec<u8>)>;

const MAX_PACKETS: usize = 100_000;

pub fn vp8_data() -> PcapData {
    load_pcap_data(include_bytes!("data/vp8.pcap"))
}

pub fn vp9_data() -> PcapData {
    load_pcap_data(include_bytes!("data/vp9.pcap"))
}

pub fn _h264_data() -> PcapData {
    load_pcap_data(include_bytes!("data/h264.pcap"))
}

pub fn load_pcap_data(data: &[u8]) -> PcapData {
    let reader = Cursor::new(data);
    let mut r = PcapReader::new(reader).expect("pcap reader");

    let exts = ExtensionMap::standard();

    let mut pcaps = vec![];

    let mut first = None;

    while let Some(pkt) = r.next_packet() {
        let pkt = pkt.unwrap();

        if first.is_none() {
            first = Some(pkt.timestamp);
        }
        let relative_time = pkt.timestamp - first.unwrap();

        // This magic number 42 is the ethernet/IP/UDP framing of the packet.
        let rtp_data = &pkt.data[42..];

        let header = RtpHeader::_parse(rtp_data, &exts).unwrap();
        let payload = &rtp_data[header.header_len..];

        pcaps.push((relative_time, header, payload.to_vec()));
    }

    pcaps.truncate(MAX_PACKETS);

    pcaps
}
