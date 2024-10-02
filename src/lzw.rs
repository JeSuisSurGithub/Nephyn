use std::io::{self, Read, Write, BufReader, BufWriter};
use ahash::AHashMap;

/*
    16-bit: 65535
    15-bit: 32767
    14-bit: 16383
    13-bit: 8191
    12-bit: 4095
*/

const CLEAR_CODE: u16 = 16383;
const MAX_DICT_SIZE: usize = CLEAR_CODE as usize;

struct TrieNode {
    snode: AHashMap<u8, TrieNode>,
    code: Option<u16>
}

impl TrieNode {
    fn new(code: Option<u16>) -> Self {
        TrieNode {
            snode: AHashMap::new(),
            code: code,
        }
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
            width: 9
        }
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
            width: 9
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

    let mut root = TrieNode::new(None);
    for i in 0..=255 {
        root.snode.insert(i as u8, TrieNode::new(Some(i as u16)));
    }
    let mut cur_code: u16 = 256;
    let mut next_growth: u32 = 512;

    let mut cur_node: &mut TrieNode = &mut root;

    // I/O

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    let mut stream = LZWStream::new();

    let mut byte = [0u8; 1];
    while reader.read_exact(&mut byte).is_ok()
    {
        if cur_node.snode.contains_key(&byte[0]) {
            cur_node = cur_node.snode.get_mut(&byte[0]).unwrap();
        } else {
            stream.write_code(
                cur_node.code.expect("[lzw_encode]: Malformed tree: missing code"),
                32,
                &mut writer)?;

            if cur_code != CLEAR_CODE {
                if (cur_code as u32) == next_growth {
                    stream.width += 1;
                    next_growth *= 2;
                }
                cur_node.snode.insert(byte[0], TrieNode::new(Some(cur_code)));
                cur_code += 1;
            } else {
                stream.write_code(
                    CLEAR_CODE,
                    32,
                    &mut writer)?;

                // Reset by trimming the first level of the tree
                for next in root.snode.values_mut() {
                    next.snode = AHashMap::new();
                }
                cur_code = 256;
                stream.width = 9;
                next_growth = 512;
            }
            cur_node = &mut root;
            cur_node = cur_node.snode.get_mut(&byte[0]).expect("[lzw_encode]: Malformed tree: badly constructed root");
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

    let mut table = vec![Vec::new(); MAX_DICT_SIZE];
    for i in 0..=255 {
        table[i] = vec![i as u8];
    }
    let mut prev_code: Option<u16> = None;
    let mut cur_code: u16 = 256;
    let mut next_growth: u32 = 511;

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
                table = vec![Vec::new(); MAX_DICT_SIZE];
                for i in 0..=255 {
                    table[i] = vec![i as u8];
                }
                cur_code = 256;
                stream.width = 9;
                next_growth = 511;
                prev_code = None;
                continue;
            }

            if code < cur_code {
                writer.write_all(&table[code as usize])?;
            } else if let Some(prev) = prev_code {
                writer.write_all(&table[prev as usize])?;
                writer.write_all(&[table[prev as usize][0]])?;
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid LZW code"));
            };

            if let Some(prev) = prev_code
            {
                let mut new_word = table[prev as usize].clone();
                if code < cur_code {
                    new_word.push(table[code as usize][0]);
                } else if let Some(prev) = prev_code {
                    new_word.push(table[prev as usize][0]);
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid LZW code"));
                };

                table[cur_code as usize] = new_word;

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