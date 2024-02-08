use super::{progress, vp8_data, vp9_data, PcapData, TestRtc};
use std::net::Ipv4Addr;
use std::time::Duration;
use str0m::format::Codec;
use str0m::media::{Direction, MediaData, MediaKind};
use str0m::{Candidate, Event, RtcError};
use tracing::info_span;

pub struct LtoR {
    codec: Codec,
    input: LtoRInput,
}

pub enum LtoRInput {
    Rtp(PcapData),
    Sample(Vec<MediaData>),
}

impl LtoR {
    pub fn with_vp8_input() -> Self {
        Self::new(Codec::Vp8, LtoRInput::Rtp(vp8_data()))
    }

    pub fn with_vp9_input() -> Self {
        Self::new(Codec::Vp9, LtoRInput::Rtp(vp9_data()))
    }

    pub fn with_samples(mds: Vec<MediaData>) -> Self {
        Self::new(Codec::Vp9, LtoRInput::Sample(mds))
    }

    pub fn new(codec: Codec, input: LtoRInput) -> Self {
        Self { codec, input }
    }

    pub fn run(&self) -> Result<(), RtcError> {
        let mut l = TestRtc::new(info_span!("L"), false);
        let mut r = TestRtc::new(info_span!("R"), false);

        let host1 = Candidate::host((Ipv4Addr::new(1, 1, 1, 1), 1000).into(), "udp")?;
        let host2 = Candidate::host((Ipv4Addr::new(2, 2, 2, 2), 2000).into(), "udp")?;
        l.add_local_candidate(host1);
        r.add_local_candidate(host2);

        // The change is on the L (sending side) with Direction::SendRecv.
        let mut change = l.sdp_api();
        let mid = change.add_media(MediaKind::Video, Direction::SendOnly, None, None);
        let (offer, pending) = change.apply().unwrap();

        let answer = r.rtc.sdp_api().accept_offer(offer)?;
        l.rtc.sdp_api().accept_answer(pending, answer)?;

        loop {
            if l.is_connected() || r.is_connected() {
                break;
            }
            progress(&mut l, &mut r)?;
        }

        let max = l.last.max(r.last);
        l.last = max;
        r.last = max;

        let params = match self.codec {
            Codec::Vp8 => l.params_vp8(),
            Codec::Vp9 => l.params_vp9(),
            _ => unimplemented!(),
        };
        assert_eq!(params.spec().codec, self.codec);

        let pt = params.pt();

        let _timespan = match &self.input {
            LtoRInput::Rtp(data) => {
                for (relative, header, payload) in data {
                    // Keep RTC time progressed to be "in sync" with the test data.
                    while (l.last - max) < *relative {
                        progress(&mut l, &mut r)?;
                    }

                    let absolute = max + *relative;

                    let mut direct = l.direct_api();
                    let tx = direct.stream_tx_by_mid(mid, None).unwrap();
                    tx.write_rtp(
                        pt,
                        header.sequence_number(None),
                        header.timestamp,
                        absolute,
                        header.marker,
                        header.ext_vals.clone(),
                        true,
                        payload.clone(),
                    )
                    .unwrap();

                    progress(&mut l, &mut r)?;
                }
                
                data.last().expect("at least one rtp data").0 - data.first().expect("at least one rtp data").0
            }
            LtoRInput::Sample(mds) => {
                let first = mds.first().expect("at least one media data");
                let timebase = first.network_time;

                for data in mds {
                    let relative = data.network_time - timebase;
                    // Keep RTC time progressed to be "in sync" with the test data.
                    while (l.last - max) < relative {
                        progress(&mut l, &mut r)?;
                    }
                    let absolute = max + relative;
                    let writer = l.writer(mid).expect("writer exists");
                    writer
                        .write(pt, absolute, data.time, data.data.clone())
                        .expect("media written");
                    progress(&mut l, &mut r).expect("wtf");
                }
                
                mds.last().expect("at least one media data").network_time - timebase
                
            }
        };

        // Settle
        let max = l.last.max(r.last) + Duration::from_secs(5);
        l.last = max;
        r.last = max;
        progress(&mut l, &mut r).expect("wtf");

        // Useful debugging
        // println!("input time span: {:?}", _timespan);
        // println!("L: {:?}, {:?}", r.media_data_count, r.duration());
        // println!("R: {:?}, {:?}", l.media_data_count, l.duration());

        Ok(())
    }

    /// here we run a pair of Rtc just to convert pcap -> MediaData
    /// so we can then use MediaData for testing non-rtp-mode
    pub fn rtp_to_mediadata(&self) -> Result<Vec<MediaData>, RtcError> {
        let mut l = TestRtc::new(info_span!("L"), false);
        let mut r = TestRtc::new(info_span!("R"), true);

        let host1 = Candidate::host((Ipv4Addr::new(1, 1, 1, 1), 1000).into(), "udp")?;
        let host2 = Candidate::host((Ipv4Addr::new(2, 2, 2, 2), 2000).into(), "udp")?;
        l.add_local_candidate(host1);
        r.add_local_candidate(host2);

        // The change is on the L (sending side) with Direction::SendRecv.
        let mut change = l.sdp_api();
        let mid = change.add_media(MediaKind::Video, Direction::SendOnly, None, None);
        let (offer, pending) = change.apply().unwrap();

        let answer = r.rtc.sdp_api().accept_offer(offer)?;
        l.rtc.sdp_api().accept_answer(pending, answer)?;

        loop {
            if l.is_connected() || r.is_connected() {
                break;
            }
            progress(&mut l, &mut r)?;
        }

        let max = l.last.max(r.last);
        l.last = max;
        r.last = max;

        let params = match self.codec {
            Codec::Vp8 => l.params_vp8(),
            Codec::Vp9 => l.params_vp9(),
            _ => unimplemented!(),
        };
        assert_eq!(params.spec().codec, self.codec);

        let pt = params.pt();

        let data = match &self.input {
            LtoRInput::Rtp(data) => data,
            _ => unimplemented!(),
        };

        for (relative, header, payload) in data {
            // Keep RTC time progressed to be "in sync" with the test data.

            while (l.last - max) < *relative {
                progress(&mut l, &mut r)?;
            }

            let absolute = max + *relative;

            let mut direct = l.direct_api();
            let tx = direct.stream_tx_by_mid(mid, None).unwrap();
            tx.write_rtp(
                pt,
                header.sequence_number(None),
                header.timestamp,
                absolute,
                header.marker,
                header.ext_vals.clone(),
                true,
                payload.clone(),
            )
            .unwrap();

            progress(&mut l, &mut r)?;
        }

        Ok(r.events
            .expect("events captured")
            .drain(..)
            .filter_map(|(_, e)| match e {
                Event::MediaData(md) => Some(md),
                _ => None,
            })
            .collect())
    }
}
