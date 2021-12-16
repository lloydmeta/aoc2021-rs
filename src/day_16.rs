use std::collections::HashMap;
use std::iter::FromIterator;
use std::result::Result as StdResult;

use anyhow::{Context, Result};
use combine::parser::char::*;
use combine::*;
use DecodedPacket::*;

pub const INPUT: &str = include_str!("../data/day_16_input");

pub fn run() -> Result<()> {
    println!("*** Day 16: Packet Decoder ***");
    println!("Input: {}", INPUT);
    let input = parse(INPUT)?;
    let decoded = input.packet.decode()?;
    let sol_1 = decoded.version_sum();
    println!("Solution 1: {:?}", sol_1);

    let sol_2 = decoded.run();
    println!("Solution 2: {:?}", sol_2);

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub struct Input {
    raw: Vec<char>,
    pub packet: Packet,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Packet(Vec<u8>);

#[derive(Debug, Eq, PartialEq)]
pub enum DecodedPacket {
    Literal {
        packet_version: usize,
        packet_type: usize,
        number: usize,
        bits_from_packet_used: usize,
    },
    OperatorType0 {
        packet_version: usize,
        packet_type: usize,
        sub_packet_bit_length: usize,
        bits_from_packet_used: usize,
        sub_packets: Vec<DecodedPacket>,
    },
    OperatorType1 {
        packet_version: usize,
        packet_type: usize,
        sub_packet_count: usize,
        bits_from_packet_used: usize,
        sub_packets: Vec<DecodedPacket>,
    },
    End,
}

impl DecodedPacket {
    pub fn packet_version(&self) -> usize {
        match self {
            Literal { packet_version, .. } => *packet_version,
            OperatorType0 { packet_version, .. } => *packet_version,
            OperatorType1 { packet_version, .. } => *packet_version,
            End => 0,
        }
    }

    pub fn version_sum(&self) -> usize {
        match self {
            Literal { packet_version, .. } => *packet_version,
            OperatorType0 {
                packet_version,
                sub_packets,
                ..
            } => {
                packet_version
                    + sub_packets
                        .iter()
                        .fold(0, |acc, next| acc + next.version_sum())
            }
            OperatorType1 {
                packet_version,
                sub_packets,
                ..
            } => {
                packet_version
                    + sub_packets
                        .iter()
                        .fold(0, |acc, next| acc + next.version_sum())
            }
            End => 0,
        }
    }

    pub fn run(&self) -> Result<usize> {
        fn run_many(packet_type: usize, packets: &[DecodedPacket]) -> Result<usize> {
            match packet_type {
                0 => {
                    let mut acc = 0;
                    for packet in packets {
                        acc += packet.run()?;
                    }
                    Ok(acc)
                }
                1 => {
                    let mut acc = 1;
                    for packet in packets {
                        acc *= packet.run()?;
                    }
                    Ok(acc)
                }
                2 => {
                    let mut lowest_result = None;
                    for packet in packets {
                        let r = packet.run()?;
                        match lowest_result {
                            Some(curr) if curr < r => (),
                            _ => lowest_result = Some(r),
                        }
                    }
                    lowest_result.context("No packets exist")
                }
                3 => {
                    let mut lowest_result = None;
                    for packet in packets {
                        let r = packet.run()?;
                        match lowest_result {
                            Some(curr) if curr > r => (),
                            _ => lowest_result = Some(r),
                        }
                    }
                    lowest_result.context("No packets exist")
                }
                5 => match (packets.get(0), packets.get(1)) {
                    (Some(first), Some(second)) => {
                        let r = if first.run()? > second.run()? { 1 } else { 0 };
                        Ok(r)
                    }
                    _ => bail!("Did not have 2 sub-packets"),
                },
                6 => match (packets.get(0), packets.get(1)) {
                    (Some(first), Some(second)) => {
                        let r = if first.run()? < second.run()? { 1 } else { 0 };
                        Ok(r)
                    }
                    _ => bail!("Did not have 2 sub-packets"),
                },
                7 => match (packets.get(0), packets.get(1)) {
                    (Some(first), Some(second)) => {
                        let r = if first.run()? == second.run()? { 1 } else { 0 };
                        Ok(r)
                    }
                    _ => bail!("Did not have 2 sub-packets"),
                },
                _ => unimplemented!(),
            }
        }

        match self {
            Literal { number, .. } => Ok(*number),
            OperatorType0 {
                packet_type,
                sub_packets,
                ..
            } => run_many(*packet_type, sub_packets),
            OperatorType1 {
                packet_type,
                sub_packets,
                ..
            } => run_many(*packet_type, sub_packets),
            End => bail!("No result for end..."),
        }
    }
}

impl Packet {
    pub fn decode(&self) -> Result<DecodedPacket> {
        Self::decode_bits(&self.0)
    }

    fn decode_bits(bits: &[u8]) -> Result<DecodedPacket> {
        let packet_length = bits.len();

        // keep skipping bits until we reach something that can be interpreted as headers

        for skipped_bits in 0..packet_length {
            let packet = &bits[skipped_bits..];
            let version_bits = &packet[0..3];
            let type_bits = &packet[3..6];
            let header_bit_count = 6;

            let packet_version_result = to_decimal(version_bits);
            let packet_type_result = to_decimal(type_bits);

            if packet_type_result.is_ok() && packet_version_result.is_ok() {
                let packet_version = to_decimal(version_bits)?;
                let packet_type = to_decimal(type_bits)?;
                match packet_type {
                    4 => {
                        // Literal
                        let mut bits_buffer = vec![];
                        for c in packet[6..].chunks_exact(5) {
                            bits_buffer.extend_from_slice(&c[1..]);
                            if c[0] == 0 {
                                break;
                            }
                        }
                        let bits_from_packet_used =
                            header_bit_count + (bits_buffer.len() / 4 * 5) + skipped_bits;
                        let number = to_decimal(bits_buffer.as_slice())?;
                        return Ok(Literal {
                            packet_version,
                            packet_type,
                            number,
                            bits_from_packet_used,
                        });
                    }
                    _ => {
                        // Operator
                        let length_type = &packet[6];
                        let bits_used_so_far = skipped_bits + header_bit_count + 1;

                        fn decode_inner_bits<F>(
                            sub_packet_bits: &[u8],
                            exit_loop_check: F,
                        ) -> Result<(Vec<DecodedPacket>, usize)>
                        where
                            F: Fn(&[DecodedPacket]) -> bool,
                        {
                            let mut sub_packets = vec![];
                            let mut sub_packet_skip = 0;

                            loop {
                                let inner_packet = &sub_packet_bits[sub_packet_skip..];

                                let interpreted_inner = Packet::decode_bits(inner_packet)?;

                                match interpreted_inner {
                                    DecodedPacket::Literal {
                                        bits_from_packet_used,
                                        ..
                                    } => {
                                        sub_packet_skip += bits_from_packet_used;
                                        sub_packets.push(interpreted_inner);
                                    }
                                    DecodedPacket::OperatorType0 {
                                        bits_from_packet_used,
                                        ..
                                    } => {
                                        sub_packet_skip += bits_from_packet_used;
                                        sub_packets.push(interpreted_inner);
                                    }
                                    DecodedPacket::OperatorType1 {
                                        bits_from_packet_used,
                                        ..
                                    } => {
                                        sub_packet_skip += bits_from_packet_used;
                                        sub_packets.push(interpreted_inner);
                                    }
                                    DecodedPacket::End => {
                                        break;
                                    }
                                }

                                if exit_loop_check(&sub_packets) {
                                    break;
                                }
                            }

                            Ok((sub_packets, sub_packet_skip))
                        }

                        match length_type {
                            0 => {
                                let length_bits = &packet[7..22];
                                let sub_packet_bit_length = to_decimal(length_bits)?;

                                let sub_packet_bits = &packet[22..22 + sub_packet_bit_length];
                                let bits_from_packet_used =
                                    bits_used_so_far + 15 + sub_packet_bit_length;

                                let (sub_packets, _) =
                                    decode_inner_bits(sub_packet_bits, |_| false)?;
                                return Ok(OperatorType0 {
                                    packet_version,
                                    packet_type,
                                    sub_packet_bit_length,
                                    bits_from_packet_used,
                                    sub_packets,
                                });
                            }
                            1 => {
                                let sub_packet_count_bits = &packet[7..18];
                                let sub_packet_count = to_decimal(sub_packet_count_bits)?;

                                let op_type_1_bits_used_so_far = bits_used_so_far + 11;
                                let sub_packet_bits = &packet[op_type_1_bits_used_so_far..];

                                let (sub_packets, sub_packet_skip) =
                                    decode_inner_bits(sub_packet_bits, |sub_packets| {
                                        sub_packets.len() == sub_packet_count
                                    })?;

                                let bits_from_packet_used =
                                    op_type_1_bits_used_so_far + sub_packet_skip;

                                return Ok(OperatorType1 {
                                    packet_version,
                                    packet_type,
                                    sub_packet_count,
                                    bits_from_packet_used,
                                    sub_packets,
                                });
                            }
                            _ => bail!("Unsupported packet type"),
                        }
                    }
                }
            } else {
                continue;
            }
        }

        Ok(End)
    }
}

fn to_decimal(bits: &[u8]) -> Result<usize> {
    let mut acc = 0;
    for (idx, next) in bits.iter().rev().enumerate() {
        if *next == 1 || *next == 0 {
            acc += *next as usize * (2usize.pow(idx as u32));
        } else {
            bail!("Nope, invalid")
        }
    }
    Ok(acc)
}

pub fn parse(s: &str) -> StdResult<Input, easy::ParseError<&str>> {
    let mapper: HashMap<char, [u8; 4]> = HashMap::from_iter([
        ('0', [0, 0, 0, 0]),
        ('1', [0, 0, 0, 1]),
        ('2', [0, 0, 1, 0]),
        ('3', [0, 0, 1, 1]),
        ('4', [0, 1, 0, 0]),
        ('5', [0, 1, 0, 1]),
        ('6', [0, 1, 1, 0]),
        ('7', [0, 1, 1, 1]),
        ('8', [1, 0, 0, 0]),
        ('9', [1, 0, 0, 1]),
        ('A', [1, 0, 1, 0]),
        ('B', [1, 0, 1, 1]),
        ('C', [1, 1, 0, 0]),
        ('D', [1, 1, 0, 1]),
        ('E', [1, 1, 1, 0]),
        ('F', [1, 1, 1, 1]),
    ]);

    let mut parser = many1(upper().or(digit())).map(|raw: Vec<char>| {
        let expanded: Vec<u8> = raw
            .iter()
            .filter_map(|c| mapper.get(c).copied())
            .flatten()
            .collect();
        let packet = Packet(expanded);
        Input { raw, packet }
    });
    let (r, _) = parser.easy_parse(s)?;
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_INPUT_1: &str = "D2FE28";
    static TEST_INPUT_OP_PACKET_1: &str = "38006F45291200";
    static TEST_INPUT_OP_PACKET_2: &str = "EE00D40C823060";
    static TEST_INPUT_OP_PACKET_3: &str = "8A004A801A8002F478";
    static TEST_INPUT_OP_PACKET_4: &str = "620080001611562C8802118E34";
    static TEST_INPUT_OP_PACKET_5: &str = "C0015000016115A2E0802F182340";
    static TEST_INPUT_OP_PACKET_6: &str = "A0016C880162017C3686B18A3D4780";

    #[test]
    fn parse_test() {
        let i = parse(TEST_INPUT_1).unwrap();
        assert_eq!(
            Input {
                raw: vec!['D', '2', 'F', 'E', '2', '8'],
                packet: Packet(vec![
                    1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0
                ])
            },
            i
        )
    }

    #[test]
    fn decode_number_test() {
        let i = parse(TEST_INPUT_1).unwrap();
        let decoded = i.packet.decode().unwrap();
        match decoded {
            Literal {
                number,
                packet_version,
                packet_type,
                bits_from_packet_used,
            } => {
                assert_eq!(2021, number);
                assert_eq!(6, packet_version);
                assert_eq!(4, packet_type);
                assert_eq!(21, bits_from_packet_used);
            }
            other => panic!("Unexpected interpreted packet {:?}", other),
        }
    }

    #[test]
    fn decode_operator_1_test() {
        let i = parse(TEST_INPUT_OP_PACKET_1).unwrap();
        let decoded = i.packet.decode().unwrap();
        match decoded {
            OperatorType0 {
                packet_version,
                packet_type,
                sub_packet_bit_length,
                bits_from_packet_used,
                sub_packets,
            } => {
                assert_eq!(1, packet_version);
                assert_eq!(6, packet_type);
                assert_eq!(27, sub_packet_bit_length);
                assert_eq!(49, bits_from_packet_used);
                assert_eq!(2, sub_packets.len());

                match &sub_packets[0] {
                    Literal {
                        number,
                        bits_from_packet_used,
                        ..
                    } => {
                        assert_eq!(10, *number);
                        assert_eq!(11, *bits_from_packet_used);
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
                match &sub_packets[1] {
                    Literal {
                        number,
                        bits_from_packet_used,
                        ..
                    } => {
                        assert_eq!(20, *number);
                        assert_eq!(16, *bits_from_packet_used);
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
            }
            other => panic!("Unexpected interpreted packet {:?}", other),
        }
    }

    #[test]
    fn decode_operator_2_test() {
        let i = parse(TEST_INPUT_OP_PACKET_2).unwrap();
        let decoded = i.packet.decode().unwrap();
        match decoded {
            OperatorType1 {
                packet_version,
                packet_type,
                sub_packet_count,
                bits_from_packet_used,
                sub_packets,
            } => {
                assert_eq!(7, packet_version);
                assert_eq!(3, packet_type);
                assert_eq!(3, sub_packet_count);
                assert_eq!(51, bits_from_packet_used);
                assert_eq!(3, sub_packets.len());

                match &sub_packets[0] {
                    Literal {
                        number,
                        bits_from_packet_used,
                        ..
                    } => {
                        assert_eq!(1, *number);
                        assert_eq!(11, *bits_from_packet_used);
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
                match &sub_packets[1] {
                    Literal {
                        number,
                        bits_from_packet_used,
                        ..
                    } => {
                        assert_eq!(2, *number);
                        assert_eq!(11, *bits_from_packet_used);
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
                match &sub_packets[2] {
                    Literal {
                        number,
                        bits_from_packet_used,
                        ..
                    } => {
                        assert_eq!(3, *number);
                        assert_eq!(11, *bits_from_packet_used);
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
            }
            other => panic!("Unexpected interpreted packet {:?}", other),
        }
    }

    #[test]
    fn decode_operator_3_test() {
        let i = parse(TEST_INPUT_OP_PACKET_3).unwrap();
        let decoded = i.packet.decode().unwrap();
        match decoded {
            OperatorType1 {
                packet_version,
                sub_packet_count,
                sub_packets,
                ..
            } => {
                assert_eq!(4, packet_version);
                assert_eq!(1, sub_packet_count);
                match &sub_packets[0] {
                    OperatorType1 {
                        packet_version,
                        sub_packet_count,
                        sub_packets,
                        ..
                    } => {
                        assert_eq!(1, *packet_version);
                        assert_eq!(1, *sub_packet_count);
                        match &sub_packets[0] {
                            OperatorType0 {
                                packet_version,
                                sub_packets,
                                ..
                            } => {
                                assert_eq!(5, *packet_version);
                                assert_eq!(1, sub_packets.len());
                                match &sub_packets[0] {
                                    Literal { packet_version, .. } => {
                                        assert_eq!(6, *packet_version);
                                    }
                                    other => panic!("Unexpected interpreted packet {:?}", other),
                                }
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
            }
            other => panic!("Unexpected interpreted packet {:?}", other),
        }
    }

    #[test]
    fn decode_operator_3_version_sum_test() {
        let i = parse(TEST_INPUT_OP_PACKET_3).unwrap();
        let decoded = i.packet.decode().unwrap();
        let version_sum = decoded.version_sum();
        assert_eq!(16, version_sum)
    }

    #[test]
    fn decode_operator_4_test() {
        let i = parse(TEST_INPUT_OP_PACKET_4).unwrap();
        let decoded = i.packet.decode().unwrap();
        match decoded {
            OperatorType1 {
                packet_version,
                sub_packet_count,
                sub_packets,
                ..
            } => {
                assert_eq!(3, packet_version);
                assert_eq!(2, sub_packet_count);
                match &sub_packets[0] {
                    OperatorType0 {
                        packet_version,
                        sub_packets,
                        ..
                    } => {
                        assert_eq!(0, *packet_version);
                        assert_eq!(2, sub_packets.len());
                        match &sub_packets[0] {
                            Literal { packet_version, .. } => {
                                assert_eq!(0, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                        match &sub_packets[1] {
                            Literal { packet_version, .. } => {
                                assert_eq!(5, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
                match &sub_packets[1] {
                    OperatorType1 {
                        packet_version,
                        sub_packet_count,
                        sub_packets,
                        ..
                    } => {
                        assert_eq!(1, *packet_version);
                        assert_eq!(2, *sub_packet_count);
                        match &sub_packets[0] {
                            Literal { packet_version, .. } => {
                                assert_eq!(0, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                        match &sub_packets[1] {
                            Literal { packet_version, .. } => {
                                assert_eq!(3, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
            }
            other => panic!("Unexpected interpreted packet {:?}", other),
        }
    }

    #[test]
    fn decode_operator_4_version_sum_test() {
        let i = parse(TEST_INPUT_OP_PACKET_4).unwrap();
        let decoded = i.packet.decode().unwrap();
        let version_sum = decoded.version_sum();
        assert_eq!(12, version_sum)
    }

    #[test]
    fn decode_operator_5_test() {
        let i = parse(TEST_INPUT_OP_PACKET_5).unwrap();
        let decoded = i.packet.decode().unwrap();
        match decoded {
            OperatorType0 {
                packet_version,
                sub_packets,
                ..
            } => {
                assert_eq!(6, packet_version);
                assert_eq!(2, sub_packets.len());
                match &sub_packets[0] {
                    OperatorType0 {
                        packet_version,
                        sub_packets,
                        ..
                    } => {
                        assert_eq!(0, *packet_version);
                        assert_eq!(2, sub_packets.len());
                        match &sub_packets[0] {
                            Literal { packet_version, .. } => {
                                assert_eq!(0, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                        match &sub_packets[1] {
                            Literal { packet_version, .. } => {
                                assert_eq!(6, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
                match &sub_packets[1] {
                    OperatorType1 {
                        packet_version,
                        sub_packet_count,
                        sub_packets,
                        ..
                    } => {
                        assert_eq!(4, *packet_version);
                        assert_eq!(2, *sub_packet_count);
                        match &sub_packets[0] {
                            Literal { packet_version, .. } => {
                                assert_eq!(7, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                        match &sub_packets[1] {
                            Literal { packet_version, .. } => {
                                assert_eq!(0, *packet_version);
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
            }
            other => panic!("Unexpected interpreted packet {:?}", other),
        }
    }

    #[test]
    fn decode_operator_5_version_sum_test() {
        let i = parse(TEST_INPUT_OP_PACKET_5).unwrap();
        let decoded = i.packet.decode().unwrap();
        let version_sum = decoded.version_sum();
        assert_eq!(23, version_sum)
    }

    #[test]
    fn decode_operator_6_test() {
        let i = parse(TEST_INPUT_OP_PACKET_6).unwrap();
        let decoded = i.packet.decode().unwrap();
        match decoded {
            OperatorType0 {
                packet_version,
                sub_packets,
                ..
            } => {
                assert_eq!(5, packet_version);
                assert_eq!(1, sub_packets.len());
                match &sub_packets[0] {
                    OperatorType1 {
                        packet_version,
                        sub_packet_count,
                        sub_packets,
                        ..
                    } => {
                        assert_eq!(1, *packet_version);
                        assert_eq!(1, *sub_packet_count);
                        match &sub_packets[0] {
                            OperatorType1 {
                                packet_version,
                                sub_packet_count,
                                sub_packets,
                                ..
                            } => {
                                assert_eq!(3, *packet_version);
                                assert_eq!(5, *sub_packet_count);

                                match &sub_packets[0] {
                                    Literal { packet_version, .. } => {
                                        assert_eq!(7, *packet_version);
                                    }
                                    other => panic!("Unexpected interpreted packet {:?}", other),
                                }
                                match &sub_packets[1] {
                                    Literal { packet_version, .. } => {
                                        assert_eq!(6, *packet_version);
                                    }
                                    other => panic!("Unexpected interpreted packet {:?}", other),
                                }
                                match &sub_packets[2] {
                                    Literal { packet_version, .. } => {
                                        assert_eq!(5, *packet_version);
                                    }
                                    other => panic!("Unexpected interpreted packet {:?}", other),
                                }
                                match &sub_packets[3] {
                                    Literal { packet_version, .. } => {
                                        assert_eq!(2, *packet_version);
                                    }
                                    other => panic!("Unexpected interpreted packet {:?}", other),
                                }
                                match &sub_packets[4] {
                                    Literal { packet_version, .. } => {
                                        assert_eq!(2, *packet_version);
                                    }
                                    other => panic!("Unexpected interpreted packet {:?}", other),
                                }
                            }
                            other => panic!("Unexpected interpreted packet {:?}", other),
                        }
                    }
                    other => panic!("Unexpected interpreted packet {:?}", other),
                }
            }
            other => panic!("Unexpected interpreted packet {:?}", other),
        }
    }

    #[test]
    fn decode_operator_6_version_sum_test() {
        let i = parse(TEST_INPUT_OP_PACKET_6).unwrap();
        let decoded = i.packet.decode().unwrap();
        let version_sum = decoded.version_sum();
        assert_eq!(31, version_sum)
    }

    #[test]
    fn run_1_test() {
        let i = parse("C200B40A82").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(3, r)
    }

    #[test]
    fn run_2_test() {
        let i = parse("04005AC33890").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(54, r)
    }

    #[test]
    fn run_3_test() {
        let i = parse("880086C3E88112").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(7, r)
    }

    #[test]
    fn run_4_test() {
        let i = parse("CE00C43D881120").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(9, r)
    }

    #[test]
    fn run_5_test() {
        let i = parse("D8005AC2A8F0").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(1, r)
    }

    #[test]
    fn run_6_test() {
        let i = parse("F600BC2D8F").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(0, r)
    }

    #[test]
    fn run_7_test() {
        let i = parse("9C005AC2F8F0").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(0, r)
    }

    #[test]
    fn run_8_test() {
        let i = parse("9C0141080250320F1802104A08").unwrap();
        let r = i.packet.decode().unwrap().run().unwrap();
        assert_eq!(1, r)
    }
}
