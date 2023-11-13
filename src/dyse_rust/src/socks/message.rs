/********************************************************************************
 *
 *      ____                     ____          __           __       _
 *     / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
 *    / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
 *   / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  )
 *  /_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/
 *        /____/
 *
 *
 *
 ********************************************************************************/

use std::time::Instant;

pub const UDP_PACKET_SIZE: usize = 1024;
pub const SOCK_HEADER_LEN: usize = 64;
pub const FRAG_HEADER_LEN: usize = 10;
pub const PAYLOAD_IDX: usize = SOCK_HEADER_LEN + FRAG_HEADER_LEN;
pub const MAX_FRAGMENT_SIZE: usize = UDP_PACKET_SIZE - PAYLOAD_IDX;

pub type UdpPacket = [u8; UDP_PACKET_SIZE];
pub type UdpPayload = Vec<u8>;

pub fn get8_bytes(idx: usize, buffer: &[u8]) -> [u8; 8] {
    [
        buffer[idx],
        buffer[idx + 1],
        buffer[idx + 2],
        buffer[idx + 3],
        buffer[idx + 4],
        buffer[idx + 5],
        buffer[idx + 6],
        buffer[idx + 7],
    ]
}

#[derive(Clone)]
pub struct MessageFragment {
    pub offset: usize,
    pub n_bytes: usize,
    pub total_fragments: usize,
    pub payload: [u8; MAX_FRAGMENT_SIZE],
}

impl MessageFragment {
    pub fn default() -> MessageFragment {
        MessageFragment {
            offset: 0,
            n_bytes: 0,
            total_fragments: 0,
            payload: [0; MAX_FRAGMENT_SIZE],
        }
    }

    pub fn new(offset: usize, n_fragments: usize) -> MessageFragment {
        MessageFragment {
            offset: offset,
            n_bytes: 0,
            total_fragments: n_fragments,
            payload: [0; MAX_FRAGMENT_SIZE],
        }
    }

    pub fn from_shatter(
        offset: usize,
        n_fragments: usize,
        mut payload: Vec<u8>,
    ) -> MessageFragment {
        let payload_size = payload.len();

        if payload_size > MAX_FRAGMENT_SIZE {
            payload.truncate(MAX_FRAGMENT_SIZE);
        } else if payload_size < MAX_FRAGMENT_SIZE {
            payload.extend(vec![0; MAX_FRAGMENT_SIZE - payload_size]);
        }

        MessageFragment {
            offset: offset,
            n_bytes: payload_size,
            total_fragments: n_fragments,
            payload: payload.try_into().unwrap(),
        }
    }

    pub fn from_bytes(buffer: UdpPacket) -> ([u8; SOCK_HEADER_LEN], MessageFragment) {
        let n_bytes = usize::from_be_bytes(get8_bytes(SOCK_HEADER_LEN + 2, &buffer));
        (
            buffer[0..SOCK_HEADER_LEN].try_into().unwrap(),
            MessageFragment {
                offset: buffer[SOCK_HEADER_LEN] as usize,
                n_bytes: n_bytes,
                total_fragments: buffer[SOCK_HEADER_LEN + 1] as usize,
                payload: buffer[PAYLOAD_IDX..UDP_PACKET_SIZE].try_into().unwrap(),
            },
        )
    }

    pub fn to_bytes(&self, header: [u8; SOCK_HEADER_LEN]) -> UdpPacket {
        let mut buffer = [0; UDP_PACKET_SIZE];
        header
            .iter()
            .chain([self.offset as u8, self.total_fragments as u8].iter())
            .chain(self.n_bytes.to_be_bytes().iter())
            .chain(self.payload.iter())
            .enumerate()
            .for_each(|(i, b)| buffer[i] = *b);
        buffer
    }
}

#[derive(Clone)]
pub struct Message {
    pub fragments: Vec<MessageFragment>,
    pub timestamp: Instant,
    pub micros_rate: u64,
    pub ntx: i64,
}

impl Message {
    pub fn new() -> Message {
        Message {
            fragments: vec![],
            timestamp: Instant::now(),
            micros_rate: u64::MAX,
            ntx: 0,
        }
    }

    pub fn shatter(payload: Vec<u8>) -> Vec<MessageFragment> {
        // let shards: Vec<&[u8]> = match payload.len() > MAX_FRAGMENT_SIZE {
        //     true => payload.chunks(MAX_FRAGMENT_SIZE).collect(),
        //     false => vec![&payload],
        // };

        let shards: Vec<&[u8]> = payload.chunks(MAX_FRAGMENT_SIZE).collect();

        (0..shards.len())
            .map(|i| MessageFragment::from_shatter(i, shards.len(), shards[i].to_vec()))
            .collect()
    }

    pub fn from_payload(payload: Vec<u8>) -> Message {
        let fragments = Message::shatter(payload);

        Message {
            fragments: fragments,
            timestamp: Instant::now(),
            micros_rate: u64::MAX,
            ntx: 0,
        }
    }

    pub fn packets(&self, header: [u8; SOCK_HEADER_LEN]) -> Vec<UdpPacket> {
        (0..self.fragments.len())
            .map(|i| self.fragments[i].to_bytes(header))
            .collect()
    }

    pub fn init_fragments(&mut self, n: usize) {
        match self.fragments.len() > 0 {
            true => (0..self.fragments.len()).for_each(|i| self.fragments[i].offset = 255),
            false => self.fragments = (0..n).map(|_| MessageFragment::new(255, n)).collect(),
        };
    }

    pub fn is_available(&self) -> bool {
        (self.micros_rate / 2) as u128 > self.timestamp.elapsed().as_micros()
            && self.micros_rate != u64::MAX
    }

    pub fn collect(&mut self, ntx: i64, micros: u64, fragment: MessageFragment) -> bool {
        if self.fragments.len() == 0 || ntx > self.ntx {
            self.init_fragments(fragment.total_fragments);
        }

        self.ntx = ntx;
        self.micros_rate = micros;
        let offset = fragment.offset;
        self.fragments[offset] = fragment;

        match (0..self.fragments.len()).find(|&i| self.fragments[i].offset != i) {
            Some(_) => false,
            None => {
                self.timestamp = Instant::now();
                true
            }
        }
    }

    pub fn to_payload(&self) -> UdpPayload {
        (0..self.fragments.len())
            .map(|i| self.fragments[i].payload[0..self.fragments[i].n_bytes].to_vec())
            .flatten()
            .collect()
    }
}
