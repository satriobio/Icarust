//! Defines code used to create the R10 signal from pore models.
use fnv::{FnvHashMap, FnvHashSet};
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use memmap2::Mmap;
use ndarray::ArrayView1;
use ndarray_npy::ViewNpyExt;
use needletail::parse_fastx_file;
use needletail::parser::SequenceRecord;
use needletail::{FastxReader, Sequence};
use nom::character::complete::{alpha1, multispace0, tab};
use nom::multi::many1;
use nom::number::complete::double;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
use rand::seq::SliceRandom;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

lazy_static! {
    static ref HASHSET: FnvHashSet<char> = {
        let mut set = FnvHashSet::default();
        set.insert('A');
        set.insert('C');
        set.insert('G');
        set.insert('T');
        set
    };
}

/// Transform a nucleic acid sequence into its "normalized" form.
///
/// The normalized form is:
///  - only AGCTN and possibly - (for gaps)
///  - strip out any whitespace or line endings
///  - lowercase versions of these are uppercased
///  - U is converted to T (make everything a DNA sequence)
///  - some other punctuation is converted to gaps
///  - IUPAC bases may be converted to N's depending on the parameter passed in
///  - everything else is considered a N
pub fn normalize(seq: &[u8]) -> Option<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::with_capacity(seq.len());
    let mut changed: bool = false;

    for n in seq.iter() {
        let (new_char, char_changed) = match (*n, false) {
            c @ (b'A', _) | c @ (b'C', _) | c @ (b'G', _) | c @ (b'T', _) => (c.0, false),
            (b'a', _) => (b'A', true),
            (b'c', _) => (b'C', true),
            (b'g', _) => (b'G', true),
            // normalize uridine to thymine
            (b't', _) | (b'u', _) | (b'U', _) => (b'T', true),
            // normalize gaps
            (b'.', _) | (b'~', _) => (b'-', true),
            // remove all whitespace and line endings
            (b' ', _) | (b'\t', _) | (b'\r', _) | (b'\n', _) => (b' ', true),
            // everything else is an N
            _ => (b' ', true),
        };
        changed = changed || char_changed;
        if new_char != b' ' {
            buf.push(new_char);
        }
    }
    if changed {
        Some(buf)
    } else {
        None
    }
}

/// hold a kmer
pub struct Kmer {
    /// The 9mer sequence
    pub sequence: String,
    /// The value for the signal of that 9mer
    pub value: f64,
}

/// Profile for sequencing
pub struct R10Settings {
    /// Digitisation to i16 I dunno
    digitisation: f64,
    /// range
    range: f64,
}

/// Simulation type - Promethion or MInion. We always use Promethion
pub enum SimType {
    /// R10
    R10,
}
// const PREFIX: &str =
//     "TTTTTTTTTTTTTTTTTTAATCAAGCAGCGGAGTTGAGGACGCGAGACGGGACTTTTTTAGCAGACTTTACGGACTACGACT";
const RANDOM_CHARS: [char; 4] = ['A', 'C', 'G', 'T'];

/// return the simulation profile for a given simulation type
pub fn get_sim_profile(sim_type: SimType) -> R10Settings {
    match sim_type {
        SimType::R10 => R10Settings {
            digitisation: 2048.0,
            range: 200.0,
        },
    }
}

/// Use nom to parse an individual line in the kmers file
pub fn parse_kmer_record(input: &str) -> IResult<&str, Kmer> {
    let (remaining, (kmer, value)) =
        separated_pair(alpha1, tab, terminated(double, multispace0))(input)?;
    let kmer = kmer.to_string();
    Ok((
        remaining,
        Kmer {
            sequence: kmer,
            value,
        },
    ))
}

/// Parse kmers and corresponding values from the file
pub fn parse_kmers(input: &str) -> IResult<&str, FnvHashMap<String, f64>> {
    let (remainder, kmer_vec) = many1(parse_kmer_record)(input)?;
    let mut kmers: HashMap<String, f64, std::hash::BuildHasherDefault<fnv::FnvHasher>> =
        FnvHashMap::default();
    for x in kmer_vec {
        kmers.insert(x.sequence, x.value);
    }
    Ok((remainder, kmers))
}

/// Generate signal for a stall sequence and adpator DNA
pub fn generate_prefix() -> Result<Vec<i16>, Box<dyn Error>> {
    let file = File::open("static/prefix.squiggle.npy").unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let view: Vec<i16> = ArrayView1::<i16>::view_npy(&mmap).unwrap().to_vec();
    Ok(view)
}

/// Replace all occurences of a character in a string with a randomly chosen A,C,G, or T.
///  Used to remove Ns from reference.
/// If char to replace is None, We actually replace anything that is not a base
fn replace_char_with_base(string: &str, char_to_replace: Option<char>) -> String {
    let mut rng: rand::rngs::ThreadRng = rand::thread_rng();

    let replaced_string: String = string
        .chars()
        .map(|c| {
            if !HASHSET.contains(&c) {
                *RANDOM_CHARS.choose(&mut rng).unwrap()
            } else {
                c
            }
        })
        .collect();

    replaced_string
}

/// Return the given number of sequences contained in a provided Fasta/Fastq file. File can be gzipped,
/// will error if not a FASTQ/FASTA file.
pub fn num_sequences<P: AsRef<Path> + std::fmt::Debug>(path: P) -> usize {
    let mut reader: Box<dyn FastxReader> =
        parse_fastx_file(&path).unwrap_or_else(|_| panic!("Can't find FASTA file at {path:#?}"));
    let mut num_seq: usize = 0;
    while reader.next().is_some() {
        num_seq += 1;
    }
    num_seq
}

/// Get the lengths of all contigs in a given FASTA/FASTQ file
pub fn sequence_lengths<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Vec<usize> {
    let mut read_lengths = vec![];
    let mut reader: Box<dyn FastxReader> =
        parse_fastx_file(&path).unwrap_or_else(|_| panic!("Can't find FASTA file at {path:#?}"));
    while let Some(record) = reader.next() {
        read_lengths.push(record.unwrap().num_bases())
    }
    read_lengths
}

/// Convert a given FASTA sequence to signal, digitising it and return a Vector of I16
pub fn convert_to_signal<'a>(
    kmers: &FnvHashMap<String, f64>,
    record: &SequenceRecord,
    profile: &R10Settings,
) -> Result<Vec<i16>, Box<dyn Error>> {
    let mut signal_vec: Vec<i16> = Vec::with_capacity(record.num_bases() * 10);
    let r: Cow<'a, [u8]> = normalize(record.sequence()).unwrap().into();
    let num_kmers = r.len() - 8;
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");
    let pb = ProgressBar::new(num_kmers.try_into().unwrap());
    pb.set_style(sty);
    for kmer in r.kmers(9) {
        let mut kmer = String::from_utf8(kmer.to_vec()).unwrap();
        kmer = replace_char_with_base(&kmer, None);
        debug!("{kmer}");
        let value = kmers.get(&kmer.to_uppercase()).unwrap_or_else(|| {
            panic!(
                "failed to retrieve value for kmer {kmer}, on contig{}",
                String::from_utf8(record.id().to_vec()).unwrap()
            )
        });
        debug!("{value}");

        let x = (value * profile.digitisation) / profile.range;
        // 10 sample for each base (sample_rate (4000) / base per second (400))
        // could also be worked out from profile.dwell_mean
        for _ in 0..10 {
            signal_vec.push(x as i16);
        }
        pb.inc(1);
    }
    pb.finish_with_message("done");

    Ok(signal_vec)
}

// read_tag, u32
// read_id
// raw_data
// daq_offset
