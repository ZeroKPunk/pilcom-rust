use std::collections::HashMap;

use super::pil_serde::Pil;

#[derive(Debug, Clone)]
pub struct DefArrayElement {
    name: String,
    id: u64,
    idx: Option<u64>,
    element_type: String,
    pol_deg: u64,
}

impl Default for DefArrayElement {
    fn default() -> Self {
        DefArrayElement {
            name: "".to_string(),
            id: 0,
            idx: Some(0),
            element_type: "".to_string(),
            pol_deg: 0,
        }
    }
}

type NameSpace = String;
type NamePol = String;
type SinglePolProxy = Vec<DefArrayElement>;
type MultiPolProxy = Vec<Vec<DefArrayElement>>;
enum PolProxy {
    Single(SinglePolProxy),
    Multi(MultiPolProxy),
}

impl PolProxy {
    fn get_multi_mut(&mut self) -> Option<&mut MultiPolProxy> {
        match self {
            PolProxy::Multi(multi) => Some(multi),
            _ => None,
        }
    }
    fn get_single_mut(&mut self) -> Option<&mut SinglePolProxy> {
        match self {
            PolProxy::Single(single) => Some(single),
            _ => None,
        }
    }
}

type PolProxyType = HashMap<NameSpace, HashMap<NamePol, PolProxy>>;
type DefType = HashMap<NameSpace, HashMap<NamePol, Vec<DefArrayElement>>>;

pub struct PolArray {
    pub n_pols: u64,
    pub def: DefType,
    pub def_array: Vec<DefArrayElement>,
    pub array: Vec<Vec<DefArrayElement>>,
    pub pol_proxy: PolProxyType,
}

impl PolArray {
    pub fn new(pil: &Pil, pol_type: String) -> Self {
        let mut n_pols = 0u64;
        if pol_type == "commit" {
            n_pols = pil.n_commitments;
        } else if pol_type == "constant" {
            n_pols = pil.n_constants;
        } else {
            panic!("Invalid pol_type: {}", pol_type);
        }

        let mut pol_proxy: PolProxyType = Default::default();

        let pil_ref = &pil.references;
        let def_array_len = pil_ref
            .iter()
            .max_by_key(|(_key, reference)| reference.id)
            .unwrap()
            .1
            .id;

        let mut def_array = vec![DefArrayElement::default(); (def_array_len + 1) as usize];
        let mut def: DefType = Default::default();
        let mut array: Vec<Vec<DefArrayElement>> =
            vec![vec![DefArrayElement::default(); 0]; def_array_len as usize];

        for ref_name in pil_ref.keys() {
            let _pil_ref = pil_ref.get(ref_name).unwrap();
            if (_pil_ref.reference_type == "cmP" && pol_type == "commit")
                || (_pil_ref.reference_type == "const" && pol_type == "constant")
            {
                if let Some((name_space, name_pol)) = ref_name.split_once('.') {
                    if _pil_ref.is_array == true {
                        for i in 0.._pil_ref.len.unwrap() {
                            // let pol_pr =
                            //     vec![DefArrayElement::default(); _pil_ref.pol_deg as usize];
                            let pol_pr: Vec<DefArrayElement> =
                                Vec::with_capacity(_pil_ref.pol_deg as usize);
                            pol_proxy
                                .entry(name_space.to_string())
                                .or_insert(HashMap::new())
                                .entry(name_pol.to_string())
                                .and_modify(|pol_proxy| {
                                    if let Some(multi) = pol_proxy.get_multi_mut() {
                                        multi.push(pol_pr.clone());
                                    }
                                })
                                .or_insert_with(|| {
                                    let mut multi = Vec::new();
                                    multi.push(pol_pr.clone());
                                    PolProxy::Multi(multi)
                                });

                            def_array[(_pil_ref.id + i) as usize] = DefArrayElement {
                                name: ref_name.to_string(),
                                id: _pil_ref.id + i,
                                idx: Some(i),
                                element_type: _pil_ref.reference_type.to_string(),
                                pol_deg: _pil_ref.pol_deg,
                            };
                            def.entry(name_space.to_string())
                                .or_insert(HashMap::new())
                                .entry(name_pol.to_string())
                                .and_modify(|def| {
                                    def[i as usize] = def_array[(_pil_ref.id + i) as usize].clone();
                                })
                                .or_insert_with(|| {
                                    let mut mut_def = vec![
                                        DefArrayElement::default();
                                        _pil_ref.len.unwrap() as usize
                                    ];
                                    mut_def[i as usize] =
                                        def_array[(_pil_ref.id + i) as usize].clone();
                                    mut_def
                                });
                            array[(_pil_ref.id + i) as usize] = pol_pr.clone();
                        }
                    } else if _pil_ref.is_array == false {
                        // let pol_pr = vec![DefArrayElement::default(); _pil_ref.pol_deg as usize];
                        let pol_pr: Vec<DefArrayElement> =
                            Vec::with_capacity(_pil_ref.pol_deg as usize);
                        pol_proxy
                            .entry(name_space.to_string())
                            .or_insert(HashMap::new())
                            .entry(name_pol.to_string())
                            .or_insert(PolProxy::Single(pol_pr.clone()));
                        def_array[_pil_ref.id as usize] = DefArrayElement {
                            name: ref_name.to_string(),
                            id: _pil_ref.id,
                            idx: None,
                            element_type: _pil_ref.reference_type.to_string(),
                            pol_deg: _pil_ref.pol_deg,
                        };
                        def.entry(name_space.to_string())
                            .or_insert(HashMap::new())
                            .entry(name_pol.to_string())
                            .or_insert(vec![def_array[_pil_ref.id as usize].clone()]);
                        array[_pil_ref.id as usize] = pol_pr.clone();
                    }
                }
            }
        }

        for i in 0..n_pols {
            if let Some(_) = def_array.get(i as usize) {
                continue;
            } else {
                panic!("Invalid pils sequence");
            }
        }

        PolArray {
            n_pols,
            def,
            def_array,
            array,
            pol_proxy,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Pil;

    #[test]
    fn test_pol_array() {
        let pil = Pil::from_json_file("test_data/pil/main.pil.json").unwrap();
        let commit_pol = super::PolArray::new(&pil, "commit".to_string());
        let constant_pol = super::PolArray::new(&pil, "constant".to_string());
    }
}
