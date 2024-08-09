#![allow(non_snake_case)]
use anyhow::Result;
use fields::field_gl::Fr as FGL;
use profiler_macro::time_profiler;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, Write};

use super::traits::FieldExtension;
use super::types::PIL;

#[derive(Default, Debug)]
pub struct PolsArray {
    pub nPols: usize,
    // nameSpace, namePol, defArray's index,
    pub def: HashMap<String, HashMap<String, Vec<usize>>>,
    pub defArray: Vec<Pol>,
    pub array: Vec<Vec<FGL>>,
    pub n: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Pol {
    pub name: String,
    pub id: usize,
    pub idx: Option<usize>,
    pub polDeg: usize,
    pub elementType: Option<String>, // "field, s8, s16, s32, s64, u16, u8"
}

#[derive(Eq, PartialEq)]
pub enum PolKind {
    Commit,
    Constant,
}

impl PolsArray {
    #[time_profiler("new_pols_array")]
    pub fn new(pil: &PIL, kind: PolKind) -> Self {
        log::trace!("Creating PolsArray for");
        let nPols = match kind {
            PolKind::Commit => pil.nCommitments,
            PolKind::Constant => pil.nConstants,
        };

        let mut def: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();
        let mut defArray: Vec<Pol> = vec![Pol::default(); nPols];
        let mut array: Vec<Vec<FGL>> = (0..nPols).map(|_| vec![FGL::default(); nPols]).collect();

        for (refName, ref_) in pil.references.iter() {
            if (ref_.type_ == "cmP" && kind == PolKind::Commit)
                || (ref_.type_ == "constP" && kind == PolKind::Constant)
            {
                let name_vec: Vec<&str> = refName.split('.').collect();
                let nameSpace = name_vec[0].to_string();
                let namePols = name_vec[1].to_string();

                if ref_.isArray {
                    let mut ns: HashMap<String, Vec<usize>> = HashMap::new();
                    let mut arrayPols: Vec<usize> = vec![0usize; ref_.len.unwrap()];
                    if def.contains_key(&nameSpace) {
                        ns.clone_from(def.get(&nameSpace).unwrap());
                        if ns.contains_key(&namePols) {
                            arrayPols.clone_from(ns.get(&namePols).unwrap());
                        }
                    }

                    for i in 0..ref_.len.unwrap() {
                        defArray[ref_.id + i] = Pol {
                            name: refName.clone(),
                            id: ref_.id + i,
                            idx: Some(i),
                            elementType: ref_.elementType.as_ref().cloned(),
                            polDeg: ref_.polDeg,
                        };
                        arrayPols[i] = ref_.id + i;
                        array[ref_.id + i] = vec![FGL::default(); ref_.polDeg];
                    }
                    ns.insert(namePols, arrayPols);
                    def.insert(nameSpace, ns);
                } else {
                    defArray[ref_.id] = Pol {
                        name: refName.clone(),
                        id: ref_.id,
                        idx: None,
                        elementType: ref_.elementType.as_ref().cloned(),
                        polDeg: ref_.polDeg,
                    };
                    let arrayPols: Vec<usize> = vec![ref_.id];
                    let mut ns: HashMap<String, Vec<usize>> = HashMap::new();
                    ns.insert(namePols, arrayPols);
                    def.insert(nameSpace, ns);
                    array[ref_.id] = vec![FGL::default(); ref_.polDeg];
                }
            }
        }

        for i in 0..nPols {
            if defArray[i].name.is_empty() {
                panic!("Invalid pils sequence");
            }
        }

        PolsArray {
            nPols: defArray.len(),
            n: defArray[0].polDeg,
            defArray,
            array,
            def,
        }
    }

    #[inline(always)]
    pub fn get(&self, pil: &PIL, ns: &String, np: &String, i: usize, j: usize) -> FGL {
        let ref_id = self.get_pol_id(pil, ns, np, i);
        self.array[ref_id][j]
    }

    /// Set the ns.np[i][j] = value, where ns is the namespace, np is the state variable, i is
    /// the i-th sub-variable of state np, and j is the i-row of np.
    ///
    /// e.g. For JS statement, constPols.Compressor.C[7][pr.row] = c[5], i is 7 and j is pr.row.
    ///
    /// Before calling this function, you must ensure that this polsarray has been initialized
    #[inline(always)]
    pub fn set_matrix(
        &mut self,
        pil: &PIL,
        ns: &String,
        np: &String,
        i: usize,
        j: usize,
        value: FGL,
    ) {
        let ref_id = self.get_pol_id(pil, ns, np, i);
        self.array[ref_id][j] = value;
    }
    #[inline(always)]
    pub fn get_pol_id(&self, pil: &PIL, ns: &String, np: &String, k: usize) -> usize {
        let pol = &pil.references[&format!("{}.{}", ns, np)];
        pol.id + k
    }

    #[time_profiler("load_cm_pols_array")]
    pub fn load(&mut self, fileName: &str) -> Result<()> {
        let mut f = File::open(fileName)?;
        let maxBufferSize = 1024 * 1024 * 256; // 256Mb
        let totalSize = self.nPols * self.n * std::mem::size_of::<FGL>();
        let metadata = fs::metadata(fileName)?;

        assert_eq!(metadata.len(), totalSize as u64, "file size not equal to expected size");

        let mut buff8: Vec<u8> = vec![0u8; std::cmp::min(totalSize, maxBufferSize)];

        let mut i = 0;
        let mut j = 0;
        let mut position = 0;
        while position < totalSize {
            log::trace!(
                "loading {:?}.. {:?} of {}",
                fileName,
                position / 1024 / 1024,
                totalSize / 1024 / 1024
            );
            let mut n = std::cmp::min(buff8.len(), totalSize - position);
            log::trace!("loading minleft {} minright {} n {} ", buff8.len(), totalSize -position, n);
            let rs = f.read(&mut buff8[..n])?;
            log::trace!(
                "read size: read size = {}, n = {}, position = {}, totalSize = {} fseek = {}",
                rs,
                n,
                position,
                totalSize,
                f.stream_position()?
            );
            if n != rs {
                log::trace!("read size not equal to expected size n: {} rs: {}", n, rs);
            }
            let buff: &[u64] = unsafe {
                std::slice::from_raw_parts(
                    buff8.as_ptr() as *const u64,
                    buff8.len() / std::mem::size_of::<u64>(),
                )
            };
            n = rs / 8;

            for l in 0..n {
                self.array[i][j] = FGL::from(buff[l]);
                i += 1;
                if i == self.nPols {
                    i = 0;
                    j += 1;
                }
            }
            position += n * 8;
        }

        Ok(())
    }

    pub fn save(&self, fileName: &str) -> Result<()> {
        let mut writer = File::create(fileName)?;
        let maxBufferSize = 1024 * 1024 * 32;
        let totalSize = self.nPols * self.n;
        let mut buff: Vec<u64> = vec![0u64; std::cmp::min(totalSize, maxBufferSize)];

        let mut p = 0usize;
        for i in 0..self.n {
            for j in 0..self.nPols {
                buff[p] = self.array[j][i].as_int() % 0xFFFFFFFF00000001; //u128
                p += 1;
                if p == buff.capacity() {
                    // copy to [u8]
                    let buff8: &[u8] = unsafe {
                        std::slice::from_raw_parts(
                            buff.as_ptr() as *const u8,
                            buff.len() * std::mem::size_of::<u64>(),
                        )
                    };
                    writer.write_all(buff8)?;
                    p = 0;
                }
            }
        }
        if p > 0 {
            let buff8: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    buff.as_ptr() as *const u8,
                    buff.len() * std::mem::size_of::<u64>(),
                )
            };
            writer.write_all(buff8)?;
        }
        Ok(())
    }

    pub fn write_buff<F: FieldExtension>(&self) -> Vec<F> {
        let mut buff: Vec<F> = vec![F::ZERO; self.n * self.nPols];
        buff.par_chunks_mut(self.nPols)
            .enumerate()
            .for_each(|(i, chunk)| {
                for j in 0..self.nPols {
                    chunk[j] = F::from(self.array[j][i]);
                }
            });
        buff
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils::types;
    use crate::utils::pil_verify::{PilVerify, Pols};
    use env_logger::{Builder, Env};
    use log::LevelFilter;

    #[test]
    fn test_load_polsarray() {

        // env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

        Builder::from_env(Env::default().default_filter_or("trace"))
        .format(|buf, record| {
            let level_color = match record.level() {
                log::Level::Error => "\x1B[31m", // Red
                log::Level::Warn => "\x1B[33m",  // Yellow
                log::Level::Info => "\x1B[32m",  // Green
                log::Level::Debug => "\x1B[36m", // Cyan
                log::Level::Trace => "\x1B[35m", // Magenta
            };
            let reset_color = "\x1B[0m";
            
            writeln!(buf,
                "{}[{}]{} {}-{}:{} - {}",
                level_color,
                record.level(),
                reset_color,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
        
        let pil = types::load_json::<PIL>("test_data/tmp/connectCheck.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("test_data/tmp/connectCheck.const").unwrap();
        let mut commit_pol = PolsArray::new(&pil, PolKind::Commit);
        commit_pol.load("test_data/tmp/connectCheck.commit").unwrap();

        // let pil = types::load_json::<PIL>("test_data/zkevm/main.pil.json").unwrap();
        // let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        // const_pol.load("test_data/zkevm/zkevm.const").unwrap();
        // let mut commit_pol = PolsArray::new(&pil, PolKind::Commit);
        // commit_pol.load("test_data/zkevm/zkevm.commit").unwrap();

        let mut pols: Pols = Default::default();

        let mut pil_verifier = PilVerify::new(&pil, &commit_pol, &const_pol, &mut pols);
        let result = pil_verifier.verify_pil();
        if result.len() > 0 {
            for r in &result {
                println!("{}", r);
            }
        }
        assert_eq!(result.len(), 0)
    }
}
