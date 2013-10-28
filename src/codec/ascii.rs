// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! 7-bit ASCII encoding.

use std::str;
use util::StrCharIndex;
use types::*;

#[deriving(Clone)]
pub struct ASCIIEncoding;

impl Encoding for ASCIIEncoding {
    fn name(&self) -> &'static str { "ascii" }
    fn encoder(&self) -> ~Encoder { ~ASCIIEncoder as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~ASCIIDecoder as ~Decoder }
}

#[deriving(Clone)]
pub struct ASCIIEncoder;

impl Encoder for ASCIIEncoder {
    fn encoding(&self) -> &'static Encoding { &ASCIIEncoding as &'static Encoding }

    fn raw_feed<'r>(&mut self, input: &'r str,
                    output: &mut ByteWriter) -> Option<EncoderError<'r>> {
        output.writer_hint(input.len());

        let mut err = None;
        for ((_,j), ch) in input.index_iter() {
            if ch <= '\u007f' {
                output.write_byte(ch as u8);
            } else {
                err = Some(CodecError {
                    remaining: input.slice_from(j),
                    problem: str::from_char(ch),
                    cause: "unrepresentable character".into_send_str(),
                });
                break;
            }
        }
        err
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<EncoderError<'static>> {
        None
    }
}

#[deriving(Clone)]
pub struct ASCIIDecoder;

impl Decoder for ASCIIDecoder {
    fn encoding(&self) -> &'static Encoding { &ASCIIEncoding as &'static Encoding }

    fn raw_feed<'r>(&mut self, input: &'r [u8],
                    output: &mut StringWriter) -> Option<DecoderError<'r>> {
        output.writer_hint(input.len());
                                        
        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                output.write_char(input[i] as char);
            } else {
                return Some(CodecError {
                    remaining: input.slice(i+1, len),
                    problem: ~[input[i]],
                    cause: "invalid sequence".into_send_str(),
                });
            }
            i += 1;
        }
        None
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<DecoderError<'static>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::ASCIIEncoding;
    use types::*;

    fn strip_cause<T,Remaining,Problem>(result: (T,Option<CodecError<Remaining,Problem>>))
                                    -> (T,Option<(Remaining,Problem)>) {
        match result {
            (processed, None) => (processed, None),
            (processed, Some(CodecError { remaining, problem, cause: _cause })) =>
                (processed, Some((remaining, problem)))
        }
    }

    macro_rules! assert_result(
        ($lhs:expr, $rhs:expr) => (assert_eq!(strip_cause($lhs), $rhs))
    )

    #[test]
    fn test_encoder() {
        let mut e = ASCIIEncoding.encoder();
        assert_result!(e.test_feed("A"), (~[0x41], None));
        assert_result!(e.test_feed("BC"), (~[0x42, 0x43], None));
        assert_result!(e.test_feed(""), (~[], None));
        assert_result!(e.test_feed("\xa0"), (~[], Some(("", ~"\xa0"))));
        assert_result!(e.test_finish(), (~[], None));
    }

    #[test]
    fn test_decoder() {
        let mut d = ASCIIEncoding.decoder();
        assert_result!(d.test_feed(&[0x41]), (~"A", None));
        assert_result!(d.test_feed(&[0x42, 0x43]), (~"BC", None));
        assert_result!(d.test_feed(&[]), (~"", None));
        assert_result!(d.test_feed(&[0xa0]), (~"", Some((&[], ~[0xa0]))));
        assert_result!(d.test_finish(), (~"", None));
    }
}

