use serde::Serialize;
use std::error::Error;
use std::fmt;
// use std::fmt::Write;
// Define the Vertex type
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Vertex(pub Vec<String>);

// Define the Fish type
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Fish(pub String);

// Define the ArcH enum corresponding to the Haskell data type
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ArcH {
    ArcH {
        is_single_child: bool,
        vertex: Vertex,
        fish: Fish,
        next: Box<ArcH>,
    },
    Single {
        is_single_child: bool,
        vertex: Vertex,
    },
    ArcHWithNewLines {
        is_single_child: bool,
        prefix: Box<ArcH>,
        children: Vec<ArcH>,
    },
    // speical case for EVAL
    EvalStatement {
        expression: String,
    },
}

#[derive(Clone, serde::Serialize)]
pub struct OriginalArcHForm {
    pub vf_pairs: Vec<(Vertex, Fish)>,
    pub last_point: Vertex,
    pub executable_expression: String,
}

// print nicely
impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Vertex(v) = self;
        write!(f, "{:?}", v)
    }
}

impl fmt::Display for Fish {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Fish(f_) = self;
        write!(f, "{}", f_)
    }
}
impl fmt::Display for OriginalArcHForm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (v, f) in self.vf_pairs.iter() {
            s.push_str(&format!("{} ><{}> ", v, f));
        }
        s.push_str(&format!("{}", self.last_point));
        write!(f, "{}", s)
    }
}

// Vec<OriginalArcHForm>
impl fmt::Debug for OriginalArcHForm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (v, f) in self.vf_pairs.iter() {
            s.push_str(&format!("{} ><{}> ", v, f));
        }
        s.push_str(&format!("{}", self.last_point));
        write!(f, "{}", s)
    }
}

pub fn convertToOriginalForm(
    prefixFromOutside: Option<OriginalArcHForm>,
    arcH: ArcH,
) -> Vec<OriginalArcHForm> {
    match arcH {
        ArcH::ArcH {
            vertex,
            fish,
            next,
            is_single_child: _,
        } => {
            let pair = (vertex, fish);

            // println!("\n\nunwrapping next: {:?}\n", next);

            let oform = convertToOriginalForm(None, *next);
            let first = oform.first().unwrap();
            let mut vf_pairs = first.vf_pairs.clone();
            vf_pairs.insert(0, pair);

            let finalResult = vec![
                (combineArcHs(
                    prefixFromOutside,
                    OriginalArcHForm {
                        vf_pairs: vf_pairs,
                        last_point: first.last_point.clone(),
                        executable_expression: "".to_string(),
                    },
                )),
            ];

            // print!("\narcH finalParsed: {:?}\n\n", finalResult);

            return finalResult;
        }
        ArcH::Single {
            vertex,
            is_single_child: _,
        } => {
            return vec![combineArcHs(
                prefixFromOutside,
                OriginalArcHForm {
                    vf_pairs: vec![],
                    last_point: vertex,
                    executable_expression: "".to_string(),
                },
            )];
        }
        ArcH::ArcHWithNewLines {
            prefix,
            children,
            is_single_child,
        } => {
            let mut results: Vec<OriginalArcHForm> = vec![];
            let mut results_with_tails: Vec<OriginalArcHForm> = vec![];
            let mut prefix_ = convertToOriginalForm(prefixFromOutside, *prefix)
                .first()
                .unwrap()
                .clone();
            for child in children {
                // println!("\n\nchild: {:?}\n", child);
                let grandchildren = convertToOriginalForm(None, child.clone());
                // check if the child is empty
                for child_ in grandchildren {
                    if (child.is_single_child()) {
                        if (results.is_empty()) {
                            prefix_ = combineArcHs(Some(prefix_), child_);
                        } else {
                            for result in results {
                                let n = combineArcHs(Some(result), child_.clone());
                                results_with_tails.push(n);
                            }
                            results = results_with_tails.clone();
                            results_with_tails = vec![];
                        }
                    } else {
                        let n = combineArcHs(Some(prefix_.clone()), child_);
                        results.push(n);
                    }
                }
            }
            return results;
        }
        ArcH::EvalStatement { expression } => {
            return vec![OriginalArcHForm {
                vf_pairs: vec![],
                last_point: Vertex(vec![]),
                executable_expression: expression,
            }];
        }
    }
}

pub fn combineArcHs(arcH1_: Option<OriginalArcHForm>, arcH2: OriginalArcHForm) -> OriginalArcHForm {
    if (arcH1_.is_none()) {
        return arcH2;
    }
    let arcH1 = arcH1_.unwrap();
    if (arcH1.last_point.0.first().unwrap().is_empty()) {
        // speical case 1: empty last point in arcH1
        let mut extended_vf_pairs = arcH1.vf_pairs.clone();
        extended_vf_pairs.extend(arcH2.vf_pairs.clone());
        return OriginalArcHForm {
            vf_pairs: extended_vf_pairs,
            last_point: arcH2.last_point,
            executable_expression: "".to_string(),
        };
    } else {
        if (!arcH2.vf_pairs.is_empty()) {
            let headV = arcH2.vf_pairs.first().unwrap().0.clone();
            let headF = arcH2.vf_pairs.first().unwrap().1.clone();
            let tail = arcH2.vf_pairs[1..].to_vec();
            if (headV.0.first().unwrap().is_empty()) {
                // speical case 2: empty first point in arcH2
                let mut extended_vf_pairs = arcH1.vf_pairs.clone();
                extended_vf_pairs.push((arcH1.last_point.clone(), headF.clone()));
                extended_vf_pairs.extend(tail.clone());
                return OriginalArcHForm {
                    vf_pairs: extended_vf_pairs,
                    last_point: arcH2.last_point,
                    executable_expression: "".to_string(),
                };
            }
        }
        // lastly, here is the default case where we connect them with an empty fish
        let mut extended_vf_pairs = arcH1.vf_pairs.clone();
        extended_vf_pairs.push((arcH1.last_point.clone(), Fish("".to_string())));
        extended_vf_pairs.extend(arcH2.vf_pairs.clone());
        return OriginalArcHForm {
            vf_pairs: extended_vf_pairs,
            last_point: arcH2.last_point,
            executable_expression: "".to_string(),
        };
    }
}

pub fn markAsSingleChild(arcH: ArcH) -> ArcH {
    let mut cloned = arcH.clone();
    match cloned {
        ArcH::ArcH {
            vertex,
            fish,
            next,
            is_single_child,
        } => {
            cloned = ArcH::ArcH {
                vertex,
                fish,
                next,
                is_single_child: true,
            };
        }
        ArcH::Single {
            vertex,
            is_single_child,
        } => {
            cloned = ArcH::Single {
                vertex,
                is_single_child: true,
            };
        }
        ArcH::ArcHWithNewLines {
            prefix,
            children,
            is_single_child,
        } => {
            cloned = ArcH::ArcHWithNewLines {
                prefix,
                children,
                is_single_child: true,
            };
        }
        _ => {}
    }
    return cloned;
}

impl ArcH {
    pub fn is_single_child(&self) -> bool {
        match self {
            ArcH::ArcH {
                is_single_child, ..
            }
            | ArcH::Single {
                is_single_child, ..
            }
            | ArcH::ArcHWithNewLines {
                is_single_child, ..
            } => *is_single_child,
            _ => true,
        }
    }
}
