#![allow(non_snake_case)]
use core::panic;
use fields::{
    field_gl::{Fr as FGL, MODULUS},
    Field, PrimeField,
};
use std::{cell::RefCell, collections::HashMap, hash::Hash};

use crate::utils::types::Reference;

use super::{
    polarray::PolsArray,
    types::{Expression, PIL},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PilReferenceWithName<T> {
    pub inner: T,
    pub name: String,
}

impl<T> PilReferenceWithName<T> {
    pub fn new(inner: T, name: String) -> Self {
        PilReferenceWithName { inner, name }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PolsArr {
    v_n: Vec<FGL>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pols {
    cm: Vec<PolsArr>,
    exps: Vec<PolsArr>,
    consts: Vec<PolsArr>,
    publics: Vec<FGL>,
    p: Vec<PolsArr>,
}

impl Default for Pols {
    fn default() -> Self {
        Pols {
            cm: vec![],
            exps: vec![],
            consts: vec![],
            publics: vec![],
            p: vec![],
        }
    }
}

pub struct PilVerify<'a> {
    pil: &'a PIL,
    cm_pols: &'a PolsArray,
    const_pols: &'a PolsArray,
    pols: &'a mut Pols,
    N: usize,
    cache_connections_map:
        RefCell<HashMap<String, HashMap<u64, HashMap<u64, HashMap<u64, (u64, u64)>>>>>,
}

impl PilVerify<'_> {
    pub fn new<'a>(
        pil: &'a PIL,
        cm_pols: &'a PolsArray,
        const_pols: &'a PolsArray,
        pols: &'a mut Pols,
    ) -> PilVerify<'a> {
        PilVerify {
            pil,
            cm_pols,
            const_pols,
            pols,
            N: cm_pols.n,
            cache_connections_map: HashMap::new().into(),
        }
    }

    pub fn get_roots(&self) -> Vec<FGL> {
        let mut roots = vec![FGL::zero(); 33];
        roots[32] = FGL::from(7277203076849721926);
        for i in (0..32).rev() {
            roots[i] = roots[i + 1] * roots[i + 1];
        }
        roots
    }

    pub fn get_Ks(&self, n: usize) -> Vec<FGL> {
        let mut ks = vec![FGL::zero(); n];
        // TODO: need to figure out wtf is this, which is `this.k = this.exp(this.nqr, 2**this.s);` in ffjavascript
        ks[0] = FGL::from(12275445934081160404);
        log::trace!("k[0]: {}", ks[0].as_int());
        for i in 1..n {
            ks[i] = ks[i - 1] * ks[0];
        }
        ks
    }

    pub fn get_connection_map(
        &self,
        nk: usize,
    ) -> HashMap<u64, HashMap<u64, HashMap<u64, (u64, u64)>>> {
        let kc =
            MODULUS.to_string() + "_" + self.N.to_string().as_str() + "_" + nk.to_string().as_str();
        log::trace!("nk: {} kc: {}", nk, kc);

        if self.cache_connections_map.borrow_mut().contains_key(&kc) {
            let borrowed_map = self.cache_connections_map.borrow_mut();
            let v = borrowed_map.get(&kc).unwrap();
            return v.clone();
        }

        let mut m: HashMap<u64, HashMap<u64, HashMap<u64, (u64, u64)>>> = HashMap::new();
        let pow = self.N.ilog2();
        let roots = self.get_roots();
        let wi = roots[pow as usize];
        let mut w = FGL::one();
        let mut ks = vec![FGL::one()];
        ks.extend(self.get_Ks(nk - 1));

        for i in 0..self.N {
            if (i % 10000) == 0 {
                log::trace!("Building cm..  {} / {}", i, self.N);
            }
            for j in 0..ks.len() {
                let a = ks[j] * w;
                let a1 = a.as_int() >> 52;
                let a2 = (a.as_int() >> 40) & 0xFFF;
                let a3 = a.as_int() & 0xFFFFFFFFFF;
                m.entry(a1)
                    .or_insert_with(HashMap::new)
                    .entry(a2)
                    .or_insert_with(HashMap::new)
                    .insert(a3, (j as u64, i as u64));
                log::trace!("insert to cm: a: {} a1: {} a2: {} a3: {} i: {} j: {} ks_len: {} ks[{}]: {} w:{}", a.as_int(), a1, a2, a3, i, j, ks.len(), j, ks[j].as_int(), w.as_int());
            }

            w = w * wi;
        }
        self.cache_connections_map
            .borrow_mut()
            .insert(kc, m.clone());
        m
    }

    pub fn eval(&self, exp: &Expression) -> Vec<FGL> {
        let mut a: Vec<FGL> = vec![];
        let mut b: Vec<FGL> = vec![];
        let mut c: FGL = FGL::zero();

        if exp.op == "add" {
            a = self.eval(&exp.values.clone().unwrap()[0]);
            b = self.eval(&exp.values.clone().unwrap()[1]);
            let mut r: Vec<FGL> = vec![FGL::zero(); a.len()];
            for i in 0..a.len() {
                r[i] = a[i] + b[i];
            }
            return r;
        } else if exp.op == "sub" {
            a = self.eval(&exp.values.clone().unwrap()[0]);
            b = self.eval(&exp.values.clone().unwrap()[1]);
            let mut r: Vec<FGL> = vec![FGL::zero(); a.len()];
            for i in 0..a.len() {
                r[i] = a[i] - b[i];
            }
            return r;
        } else if exp.op == "mul" {
            a = self.eval(&exp.values.clone().unwrap()[0]);
            b = self.eval(&exp.values.clone().unwrap()[1]);
            let mut r: Vec<FGL> = vec![FGL::zero(); a.len()];
            for i in 0..a.len() {
                r[i] = a[i] * b[i];
            }
            return r;
        } else if exp.op == "addc" {
            a = self.eval(&exp.values.clone().unwrap()[0]);
            c = FGL::from(exp.const_.unwrap() as u64);
            let mut r: Vec<FGL> = vec![FGL::zero(); a.len()];
            for i in 0..a.len() {
                r[i] = a[i] + c;
            }
            return r;
        } else if exp.op == "mulc" {
            a = self.eval(&exp.values.clone().unwrap()[0]);
            c = FGL::from(exp.const_.unwrap() as u64);
            let mut r: Vec<FGL> = vec![FGL::zero(); a.len()];
            for i in 0..a.len() {
                r[i] = a[i] * c;
            }
            return r;
        } else if exp.op == "neg" {
            a = self.eval(&exp.values.clone().unwrap()[0]);
            let mut r: Vec<FGL> = vec![FGL::zero(); a.len()];
            for i in 0..a.len() {
                r[i] = -a[i];
            }
            return r;
        } else if exp.op == "cm" {
            let mut r: Vec<FGL> = self.pols.cm[exp.id.unwrap()].v_n.clone();
            if exp.next() {
                r = self.get_prime(&r)
            };
            return r;
        } else if exp.op == "const" {
            let mut r: Vec<FGL> = self.pols.consts[exp.id.unwrap()].v_n.clone();
            if exp.next() {
                r = self.get_prime(&r)
            };
            return r;
        } else if exp.op == "exp" {
            let mut r: Vec<FGL> = self.pols.exps[exp.id.unwrap()].v_n.clone();
            if exp.next() {
                r = self.get_prime(&r)
            };
            return r;
        } else if exp.op == "number" {
            let v = FGL::from(exp.value.clone().unwrap().parse::<u64>().unwrap());
            let mut r: Vec<FGL> = vec![FGL::zero(); self.N];
            for i in 0..self.N {
                r[i] = v;
            }
            return r;
        } else if exp.op == "public" {
            let mut r: Vec<FGL> = vec![FGL::zero(); self.N];
            for i in 0..self.N {
                r[i] = self.pols.publics[exp.id.unwrap()];
            }
            return r;
        } else {
            panic!("Unknown operation: {}", exp.op);
        }
    }

    pub fn get_prime(&self, p: &Vec<FGL>) -> Vec<FGL> {
        let mut r = p[1..].to_vec();
        r[p.len() - 1] = p[0];
        r
    }

    pub fn calculate_expressions(&mut self, exp_id: usize) -> Vec<FGL> {
        log::trace!("Calculating expressions {}", exp_id);
        if self.pols.exps.len() - 1 >= exp_id && self.pols.exps[exp_id].v_n.len() > 0 {
            return self.pols.exps[exp_id].v_n.clone();
        }
        self.calculate_dependencies(&self.pil.expressions[exp_id]);
        let p = self.eval(&self.pil.expressions[exp_id]);

        self.pols.exps[exp_id].v_n = p.clone();
        return self.pols.exps[exp_id].v_n.clone();
    }

    pub fn calculate_dependencies(&mut self, exp: &Expression) {
        if exp.op == "exp" {
            self.calculate_expressions(exp.id.unwrap());
        }
        if exp.values.is_some() {
            for i in 0..exp.values.clone().unwrap().len() {
                self.calculate_dependencies(&exp.values.clone().unwrap()[i]);
            }
        }
    }

    pub fn verify_pil(&mut self) -> Vec<String> {
        let mut ref_cm: HashMap<usize, PilReferenceWithName<Reference>> = HashMap::new();
        let mut ref_const: HashMap<usize, PilReferenceWithName<Reference>> = HashMap::new();
        let mut ref_im: HashMap<usize, PilReferenceWithName<Reference>> = HashMap::new();
        let mut res: Vec<String> = vec![];

        for (refName, ref_) in self.pil.references.iter() {
            let ref_with_name = PilReferenceWithName::new(ref_.clone(), refName.clone());
            if ref_.type_ == "cmP" {
                ref_cm.insert(ref_.id, ref_with_name);
            } else if ref_.type_ == "constP" {
                ref_const.insert(ref_.id, ref_with_name);
            } else if ref_.type_ == "imP" {
                ref_im.insert(ref_.id, ref_with_name);
            } else {
                panic!("Unknown type of reference: {}", ref_.type_);
            }
        }

        self.pols.cm = vec![PolsArr { v_n: vec![] }; self.pil.nCommitments];
        self.pols.exps = vec![PolsArr { v_n: vec![] }; self.pil.expressions.len()];
        self.pols.consts = vec![PolsArr { v_n: vec![] }; self.pil.nConstants];

        // 1.- Prepare commited polynomials.
        for i in 0..self.cm_pols.nPols {
            self.pols.cm[i].v_n = self.cm_pols.array[i].clone();
        }
        for i in 0..self.const_pols.nPols {
            self.pols.consts[i].v_n = self.const_pols.array[i].clone();
        }
        self.pols.publics = vec![FGL::zero(); self.pil.publics.len()];
        // self.pols.exps

        for i in 0..self.pil.publics.len() {
            log::trace!("preparing public {} / {}", i, self.pil.publics.len());
            if self.pil.publics[i].polType == "cmP" {
                self.pols.publics[i] =
                    self.pols.cm[self.pil.publics[i].polId].v_n[self.pil.publics[i].idx].clone();
            } else if self.pil.publics[i].polType == "imP" {
                self.calculate_expressions(self.pil.publics[i].polId);
                self.pols.publics[i] =
                    self.pols.exps[self.pil.publics[i].polId].v_n[self.pil.publics[i].idx].clone();
                self.pols.exps[self.pil.publics[i].polId].v_n = vec![];
            } else {
                panic!("Unknown public type: {}", self.pil.publics[i].polType);
            }
        }

        for i in 0..self.pil.connectionIdentities.clone().unwrap().len() {
            log::trace!(
                "Checking connectionIdentities {} / {}",
                i + 1,
                self.pil.connectionIdentities.clone().unwrap().len()
            );
            let ci = self.pil.connectionIdentities.clone().unwrap()[i].clone();

            let ci_pols = ci.pols.clone().unwrap();
            let ci_cons = ci.connections.clone().unwrap();

            for j in 0..ci_pols.len() {
                self.calculate_expressions(ci_pols[j]);
            }
            for j in 0..ci_cons.len() {
                self.calculate_expressions(ci_cons[j]);
            }
            log::trace!("start generating cm");
            let cm = self.get_connection_map(ci_pols.len());
            log::trace!("cm {:?}", cm);

            for mut j in 0..ci_pols.len() {
                for mut k in 0..self.N {
                    if k % 10000 == 0 {
                        log::trace!("{} / {}", k+1, self.N);
                    }
                    let v1 = self.pols.exps[ci_pols[j]].v_n[k].as_int();
                    let a = self.pols.exps[ci_cons[j]].v_n[k].as_int();
                    let a1 = a >> 52;
                    let a2 = (a >> 40) & 0xFFF;
                    let a3 = a & 0xFFFFFFFFFF;
                    log::debug!(
                        "{:?}:{}: a1={} a2={} a3={} a={}",
                        ci.fileName.to_string(),
                        ci.line,
                        a1,
                        a2,
                        a3,
                        a
                    );
                    let get_res = cm.get(&a1).unwrap().get(&a2).unwrap().get(&a3);
                    
                    match get_res {
                        Some((cp, cw)) => {
                            log::debug!("cp={} cw={} a1={} a2={} a3={}", cp, cw, a1, a2, a3);
                            let v2 =
                                self.pols.exps[ci_pols[*cp as usize]].v_n[*cw as usize].as_int();
                            if FGL::from(v1) != FGL::from(v2) {
                                let log_str = format!("{:?}:{:?}: connection does not match p1={:?} w1={:?} p2={:?} w2={:?} val= {} != {}",ci.fileName, self.pil.connectionIdentities.clone().unwrap()[i].line,j,k,cp,cw,v1,v2);
                                log::error!("{}", log_str);
                                k = self.N;
                                j = ci_pols.len();
                            }
                        }
                        None => {
                            let log_str = format!(
                                "{:?}:{:?}: invalid copy value w={:?},{:?} val={:?}",
                                ci.fileName,
                                self.pil.connectionIdentities.clone().unwrap()[i].line,
                                j,
                                k,
                                v1
                            );
                            log::error!("{}", log_str);
                            res.push(format!("{}", log_str));
                        }
                    }
                }
            }
            for j in 0..ci_pols.len() {
                self.pols.exps[ci_pols[j]].v_n = vec![];
            }
            for j in 0..ci_cons.len() {
                self.pols.exps[ci_cons[j]].v_n = vec![];
            }
        }

        res
    }
}
