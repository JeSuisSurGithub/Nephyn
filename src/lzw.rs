use std::io::{self, Read, Write, BufReader, BufWriter};

/*
    16-bit: 65535
    15-bit: 32767
    14-bit: 16383
    13-bit: 8191
    12-bit: 4095
*/

const CLEAR_CODE: u16 = 16383;
const MAX_DICT_SIZE: usize = CLEAR_CODE as usize;

const INIT_BIT_LEN: u8 = 9;
const INIT_CODE: u16 = 256;
const INIT_ENCODE_GROWTH_TRIGGER: u32 = 512;
const INIT_DECODE_GROWTH_TRIGGER: u32 = 511;

const WRITE_FLUSH_THRESHOLD: u8 = 48;

struct TrieNode {
    snode: Vec<Option<TrieNode>>,
    byte: Option<u8>,
    code: Option<u16>,
    // packed256_truth_table: [u64; 4],
}

impl TrieNode {
    fn new(code: Option<u16>, byte: Option<u8>) -> Self {
        TrieNode {
            snode: Vec::new(),
            byte: byte,
            code: code,
            // packed256_truth_table: [0; 4],
        }
    }

    fn insert(&mut self, byte: u8, code: u16) {
        self.snode.push(Some(TrieNode::new(Some(code), Some(byte))));
        // self.packed256_truth_table[(byte / 64) as usize] |= 1 << (byte % 64);
    }

    fn exists(&self, byte: u8) -> Result<usize, ()> {
        // if (self.packed256_truth_table[(byte / 64) as usize] & (1 << (byte % 64))) == 0 {
        //     return Err(());
        // }
        for (i, node) in self.snode.iter().enumerate() {
            if let Some(next_) = &*node {
                if next_.byte.unwrap() == byte {
                    return Ok(i);
                }
            }
        }
        return Err(());
    }

    fn next(&mut self, byte: u8) -> &mut TrieNode {
        return self.snode[byte as usize].as_mut().expect("[lzw_encode]: Bad heuristics or malformed structure");
    }

    fn trim(&mut self) {
        for next in self.snode.iter_mut() {
            if let Some(next_) = next {
                next_.snode = Vec::new();
            }
        }
    }
}

struct FlatPrefixChain {
    table: [(u8, Option<u16>); MAX_DICT_SIZE]
}

impl FlatPrefixChain {
    fn new() -> Self {
        let mut chain = FlatPrefixChain {
            table: [(0u8, None) ; MAX_DICT_SIZE]
        };
        for i in 0..=255 {
            chain.table[i] = (i as u8, None);
        }
        return chain;
    }

    fn build(&mut self, buf: &mut Vec<u8>, code: u16) {
        let mut idx = code as usize;
        loop {
            let (byte, parent) = self.table[idx];
            buf.insert(0, byte);
            if let Some(next_idx) = parent {
                idx = next_idx as usize;
            } else {
                break;
            }
        }
    }

    fn add(&mut self, code: u16, byte: u8, prev: u16) {
        self.table[code as usize] = (byte, Some(prev));
    }
}

struct LZWStream {
    buf: u64,
    len: u8,
    width: u8,
}

impl LZWStream {
    fn new() -> Self {
        LZWStream {
            buf: 0,
            len: 0,
            width: INIT_BIT_LEN
        }
    }

    fn grow(&mut self) {
        self.width += 1;
    }

    fn reset_width(&mut self) {
        self.width = INIT_BIT_LEN;
    }

    fn write_code(&mut self, code: u16, threshold: u8, writer: &mut BufWriter<&mut dyn Write>) -> Result<(), io::Error>
    {
        self.buf = (self.buf << self.width) | code as u64;
        self.len += self.width;

        if self.len > threshold
        {
            while self.len >= 8 {
                writer.write_all(&[(self.buf >> (self.len - 8)) as u8])?;
                self.len -= 8;
            }
        }
        Ok(())
    }

    fn flush(&self, writer: &mut BufWriter<&mut dyn Write>) -> Result<(), io::Error>
    {
        if self.buf > 0 {
            writer.write_all(&[(self.buf << (8 - self.len)) as u8])?;
        }
        Ok(())
    }
}

struct LZWStream2 {
    buf: u64,
    len: u8,
    width: u8,
}

impl LZWStream2 {
    fn new() -> LZWStream2 {
        LZWStream2 {
            buf: 0,
            len: 0,
            width: INIT_BIT_LEN
        }
    }

    fn add_byte(&mut self, byte: u8) {
        self.buf = (self.buf << 8) | byte as u64;
        self.len += 8;
    }

    fn get_code(&mut self) -> Option<u16> {
        if self.len >= self.width {
            let code = (self.buf >> (self.len - self.width)) as u16;
            self.len -= self.width;
            self.buf &= (1 << self.len) - 1;
            Some(code)
        } else {
            None
        }
    }
}

pub fn lzw_encode(input: &mut dyn Read, output: &mut dyn Write) -> Result<(), io::Error>
{
    // LZW Data

    let mut root = TrieNode::new(None, None);
    for i in 0..=255 {
        root.insert(i as u8, i as u16);
    }
    let mut cur_code: u16 = INIT_CODE;
    let mut next_growth: u32 = INIT_ENCODE_GROWTH_TRIGGER;

    let mut cur_node = &mut root;

    // I/O

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    let mut stream = LZWStream::new();

    let mut byte = [0u8; 1];
    while reader.read_exact(&mut byte).is_ok()
    {
        match cur_node.exists(byte[0]) {
            Ok(idx) => {
                // It exists for sure
                cur_node = cur_node.next(idx as u8);
            }
            Err(_) => {
                stream.write_code(
                    cur_node.code.expect("[lzw_encode]: Malformed tree: missing code"),
                    WRITE_FLUSH_THRESHOLD,
                    &mut writer)?;

                if cur_code != CLEAR_CODE {
                    if (cur_code as u32) == next_growth {
                        stream.grow();
                        next_growth *= 2;
                    }
                    cur_node.insert(byte[0], cur_code as u16);
                    cur_code += 1;
                } else {
                    stream.write_code(
                        CLEAR_CODE,
                        WRITE_FLUSH_THRESHOLD,
                        &mut writer)?;

                    root.trim();
                    cur_code = INIT_CODE;
                    stream.reset_width();
                    next_growth = INIT_ENCODE_GROWTH_TRIGGER;
                }
                // It exists for sure
                cur_node = &mut root;
                cur_node = cur_node.next(byte[0]);
            }
        }
    }

    if cur_node.code.is_some() {
        stream.write_code(
            cur_node.code.unwrap(),
            0,
            &mut writer)?;
    }

    stream.flush(&mut writer)?;
    writer.flush()?;
    Ok(())
}

pub fn lzw_decode(input: &mut dyn Read, output: &mut dyn Write) -> io::Result<()>
{
    // LZW
    let mut table = FlatPrefixChain::new();
    let mut prev_code: Option<u16> = None;
    let mut cur_code: u16 = INIT_CODE;
    let mut next_growth: u32 = INIT_DECODE_GROWTH_TRIGGER;

    // I/O

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);
    let mut stream = LZWStream2::new();
    let mut byte = [0u8; 1];

    while reader.read_exact(&mut byte).is_ok() {
        stream.add_byte(byte[0]);

        while let Some(code) = stream.get_code()
        {
            if code == CLEAR_CODE {
                // Table is never used for checking so no resetting
                cur_code = INIT_CODE;
                stream.width = INIT_BIT_LEN;
                next_growth = INIT_DECODE_GROWTH_TRIGGER;
                prev_code = None;
                continue;
            }

            let code_exists =  code < cur_code;

            let mut buf: Vec<u8> = Vec::new();
            if code_exists {
                table.build(&mut buf, code);
                writer.write_all(buf.as_slice())?;
            } else if let Some(prev) = prev_code {
                table.build(&mut buf, prev);
                writer.write_all(buf.as_slice())?;
                writer.write_all(&[buf[0]])?;
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid LZW code"));
            }

            if let Some(prev) = prev_code
            {
                if code_exists {
                    table.add(cur_code, buf[0], prev);
                } else if let Some(prev) = prev_code {
                    table.add(cur_code, buf[0], prev);
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid LZW code"));
                }

                if (cur_code as u32) == next_growth {
                    stream.width += 1;
                    next_growth = 2u32.pow(stream.width as u32) - 1;
                }

                cur_code += 1;
            }
            prev_code = Some(code);
        }
    }

    writer.flush()?;
    Ok(())
}