use serde::Deserialize;

use std::fs::File;
use std::collections::HashMap;
use std::fmt::Display;
use std::env::args;
use average::Mean;

fn main() {
    File::open("materia.json")
        .map_err(|e| format!("Unable to open file: {:?}", e))
        .and_then(|file| {
            let a: Result<Materia, serde_json::Error> = serde_json::from_reader(file);
            a
                .map_err(|e| format!("Unable to read/parse: {:?}", e))
        })
        .and_then(|input_materia| {
            match args().skip(1).next() {
                Some(val) => {
                    match val.as_str() {
                        "--78" => {
                            operate_78(&input_materia);
                            Ok(())
                        },
                        "--transmute" => {
                            match args().skip(2).next() {
                                Some(input_trans) => {
                                    operate_specific(&input_materia, input_trans)
                                },
                                None => {
                                    Err(String::from("Invalid access"))
                                }
                            }
                        },
                        "--all" => {
                            operate_all(&input_materia);
                            Ok(())
                        }
                        _ => {
                            Err(format!("Unknown arg {}", val))
                        }
                    }
                },
                None => {
                    for grade in input_materia.grades {
                        operate_basic(&grade.materia, grade.name);
                        println!("\n");
                    }

                    Ok(())
                }
            }
        })
        .unwrap_or_else(|err| eprintln!("{}", err));

}

struct TransmuteSet<'a> {
    m1: &'a String,
    m2: &'a String,
    m3: &'a String,
    m4: &'a String,
    m5: &'a String,
}

impl<'a> TransmuteSet<'a> {
    pub fn to_string_vec(&self) -> Vec<String> {
        vec![self.m1.clone(),
             self.m2.clone(),
             self.m3.clone(),
             self.m4.clone(),
             self.m5.clone()]
    }
}

impl<'a> std::fmt::Debug for TransmuteSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {}, {})", self.m1, self.m2, self.m3, self.m4, self.m5)
    }
}

fn operate_all(materia: &Materia) {
    let mut all_combos = HashMap::<String, Vec<TransmuteSet>>::new();
    let m_sets = materia.grades.iter()
        .map(|a| (a.name.clone(), a.materia.keys().collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    for m_set in m_sets {
        let mut trans_sets = Vec::with_capacity(371293);
//        all_combos.insert(m_set.0.clone(), Vec::with_capacity(371293));
        let mut rec_set: [usize; 5] = [0, 0, 0, 0, 0];
        while rec_set[0] < 13 {
            rec_set[1] = 0;
            rec_set[2] = 0;
            rec_set[3] = 0;
            rec_set[4] = 0;
            while rec_set[1] < 13 {
                rec_set[2] = 0;
                rec_set[3] = 0;
                rec_set[4] = 0;
                while rec_set[2] < 13 {
                    rec_set[3] = 0;
                    rec_set[4] = 0;
                    while rec_set[3] < 13 {
                        rec_set[4] = 0;
                        while rec_set[4] < 13 {
                            let m1 = m_set.1[rec_set[0]];
                            let m2 = m_set.1[rec_set[1]];
                            let m3 = m_set.1[rec_set[2]];
                            let m4 = m_set.1[rec_set[3]];
                            let m5 = m_set.1[rec_set[4]];
                            trans_sets.push(TransmuteSet{m1, m2, m3, m4, m5});

                            rec_set[4] += 1;
                        }
                        rec_set[3] += 1;
                    }
                    rec_set[2] += 1;
                }
                rec_set[1] += 1;
            }
            rec_set[0] += 1;
        }
        all_combos.insert(m_set.0, trans_sets);
    }

    for m_set in &materia.grades {
        let combo_grade = &all_combos[&m_set.name];
        let mut max_profit: i64 = i64::min_value();
        let mut max_profit_combo: Option<&TransmuteSet> = None;
        let mut max_profit_result: Option<TransmuteResult> = None;
        let map = &m_set.materia;
        for combo in combo_grade {
            let result = operate_specific_materia(map, combo.to_string_vec());

            let profit = result.others_average as i64 - result.input_sum as i64;
            if max_profit < profit {
                max_profit = profit;
                max_profit_combo = Some(combo);
                max_profit_result = Some(result);
            }
        }
        println!("Max profit for grade {}:", &m_set.name);
        match max_profit_combo {
            Some(combo) => {
                let result = max_profit_result.unwrap();
                println!("{:?}", combo);
                println!("Cost: {}", result.input_sum);
                println!("Average Result: {}", result.others_average);
                println!("Average Profit: {}", result.others_average as i64 - result.input_sum as i64);
                println!("Cost-Profit Ratio: {:.3}", result.others_average as f64 / result.input_sum as f64);
            },
            None => {
                println!("No profit for grade {}!", &m_set.name);
            }
        }
        println!("--------------------------------");
        println!("--------------------------------");
    }

}

fn operate_specific(materia: &Materia, trans: String) -> Result<(), String> {
    let input: Vec<&str> = trans.split(",").collect();
    let input: Vec<String> = input.into_iter().map(|a| String::from(a)).collect();
    if input.len() != 5 {
        Err(String::from("Invalid input for transmutation"))
    } else {
        let working_grade = materia.grades.iter()
            .filter(|grade| input.iter().all(|mat| grade.materia.contains_key(mat)))
            .next();
        let result = working_grade.ok_or(String::from("Invalid materia -- Grade incompatible or not found"));
        result.and_then(|materia_tier| {
            let transmute = operate_specific_materia(&materia_tier.materia, input);
            println!("Sum of input: {}", transmute.input_sum);
            println!("Average of others: {}", transmute.others_average);
            Ok(())
        })
    }
}

#[derive(Copy, Clone)]
struct TransmuteResult {
    pub input_sum: u32,
    pub others_average: u32,
}

fn operate_specific_materia(materia: &HashMap<String, u32>, trans: Vec<String>) -> TransmuteResult {
    let sum_cost = trans.iter().fold(0u32, |a,x| a + materia[x]);
    let others = materia.iter()
        .filter(|f| !trans.contains(f.0))
        .map(|a| (a.0.clone(), *a.1))
        .collect::<HashMap<_,_>>();
    let others_average = average(&others);

    TransmuteResult{input_sum: sum_cost, others_average}

}

fn smallest_five(mats: &HashMap<String, u32>) -> HashMap<String, u32> {
    let mut a: Vec<(String, u32)> = mats.iter().map(|a| (a.0.clone(), a.1.clone())).collect();
    a.sort_by(|v0, v1| v0.1.cmp(&v1.1));
    let b: HashMap<String, u32> = a[0..5].iter().cloned().collect();
    b
}


fn operate_78(materia: &Materia) {
    let materia_7 = &materia.grades.iter().filter(|a| a.name == "VII").next().expect("No grade 7").materia;
    let materia_8 = &materia.grades.iter().filter(|a| a.name == "VIII").next().expect("No grade 8").materia;

    let e26_8avg = average(materia_8);
    let f26_8avgdiv5 = e26_8avg / 5;
    let f28_8avgdiv4 = e26_8avg / 4;
    let e28_8cutoff = (0.92 * f26_8avgdiv5 as f64) as u32;
    let b26_7avg = average(materia_7);
    let c26_7avgdiv5 = b26_7avg / 5;
    let c28_7avgdiv4 = b26_7avg / 4;
    let b28_7cutoff = (0.92 * c26_7avgdiv5 as f64 * 0.92 + 0.92 * 0.08 * f26_8avgdiv5 as f64) as u32;

    let (sell_7, mut buy_or_trans_7): (Vec<(_,_)>, Vec<(_,_)>) = materia_7
        .iter().map(|a| (a.0.clone(), *a.1))
        .partition(|a| a.1 >= c28_7avgdiv4 * 2 );
    let (sell_8, mut buy_or_trans_8): (Vec<(_,_)>, Vec<(_,_)>) = materia_8
        .iter().map(|a| (a.0.clone(), *a.1))
        .partition(|a| a.1 >= f28_8avgdiv4 * 2 );

    let (buy_7, trans_7): (Vec<(_,_)>, Vec<(_,_)>) = buy_or_trans_7
        .iter().cloned()
        .partition(|a| a.1 < b28_7cutoff);

    let (buy_8, trans_8): (Vec<(_,_)>, Vec<(_,_)>) = buy_or_trans_8
        .iter().cloned()
        .partition(|a| a.1 < e28_8cutoff);

    buy_or_trans_7.sort_by(|a, b| a.1.cmp(&b.1));
    buy_or_trans_8.sort_by(|a, b| a.1.cmp(&b.1));

    let to_trans_7 = fill_transmute_vec(&buy_or_trans_7);
    let to_trans_8 = fill_transmute_vec(&buy_or_trans_8);

    let grade_7_sell_avg = average(&sell_7);
    let grade_8_average = average(materia_8);
    let g_7_incl_8 = (grade_7_sell_avg as f64 * 0.92 + grade_8_average as f64 * 0.08) as u32;

    println!("Grade 7 Transmute:");
    println!("----------------------");
    to_trans_7.print_pair_set();
    println!();
    println!("Sum of 7 to transmute: {}", to_trans_7.sum());
    println!("Mean of remaining 7: {}", average(&sell_7));
    println!("Mean of remaining 7 incl. chance of 8: {}", g_7_incl_8);
    println!("----------------------");
    println!("Grade 7 Sell:");
    sell_7.print_pair_set();
    println!("----------------------");
    println!("Grade 7 Transmute but do not buy:");
    trans_7.print_pair_set();
    println!("----------------------");
    println!("Grade 7 Buy (and Transmute)");
    buy_7.print_pair_set();
    println!("Grade 7 Buying cutoff: {}", b28_7cutoff);

    println!();
    println!();
    println!("=======================");
    println!("=======================");
    println!("=======================");
    println!();
    println!();

    println!("Grade 8 Transmute:");
    println!("----------------------");
    to_trans_8.print_pair_set();
    println!();
    println!("Sum of 8 to transmute: {}", to_trans_8.sum());
    println!("Mean of remaining 8: {}", average(&sell_8));
    println!("----------------------");
    println!("Grade 8 Sell:");
    sell_8.print_pair_set();
    println!("----------------------");
    println!("Grade 8 Transmute but do not buy:");
    trans_8.print_pair_set();
    println!("----------------------");
    println!("Grade 8 Buy (and Transmute)");
    buy_8.print_pair_set();
    println!("Grade 8 Buying cutoff: {}", e28_8cutoff);


}

fn fill_transmute_vec(input: &Vec<(String, u32)>) -> Vec<(String, u32)> {
    let mut to_trans = Vec::with_capacity(5);
    if input.len() > 0 {
        for (i, v) in input.iter().cloned().enumerate() {
            if i < 5 {
                to_trans.push(v);
            }
        }
        while to_trans.len() < 5 {
            to_trans.push(input[0].clone())
        }
    }
    to_trans
}

fn operate_basic<S>(set: &HashMap<String, u32>, grade: S) where S: Display {
    let sm5_7 = smallest_five(set);
    println!("Cheapest 5 Grade {} Materia: \n", grade);
    sm5_7.print_pair_set();

    let others_7: HashMap<String, u32> =
        set.iter()
            .filter(|a| !sm5_7.contains_key(a.0))
            .map(|a| (a.0.clone(), a.1.clone()))
            .collect();


    let sum7: u32 = sm5_7.sum();
    let avg_o7 = average(&others_7);
    println!("\nCost to input smallest 5: {}", sum7);
    println!("Average of remaining materia: {}", avg_o7);
}

trait DisplayPair {
    fn print_pair_set(&self);
}

impl<K,V> DisplayPair for Vec<(K,V)>
    where K: Display, V: Display
{
    fn print_pair_set(&self) {
        self.iter().for_each(|a| println!("{} @ {}", a.0, a.1));
    }
}

impl<K,V,R> DisplayPair for HashMap<K,V,R>
    where K: Display, V: Display
{
    fn print_pair_set(&self) {
        self.iter().for_each(|a| println!("{} @ {}", a.0, a.1));
    }
}

trait Summable {
    fn sum(&self) -> u32;
}

impl<K> Summable for Vec<(K, u32)> {
    fn sum(&self) -> u32 {
        self.iter().map(|a| a.1).sum()
    }
}

impl<K> Summable for HashMap<K, u32> {
    fn sum(&self) -> u32 {
        self.values().sum()
    }
}

fn average<P: ToMean>(a: &P) -> u32 {
    a.to_mean().mean() as u32
}

trait ToMean {
    fn to_mean(&self) -> Mean;
}

impl<K, V> ToMean for Vec<(K, V)>
    where V: Into<f64> + Copy
{
    fn to_mean(&self) -> Mean {
        self.iter().map(|a| a.1).map(|a| { let b: f64 = a.into(); b }).collect()
    }
}

impl<K, V> ToMean for HashMap<K, V>
    where V: Into<f64> + Copy + Clone
{
    fn to_mean(&self) -> Mean {
        self.values().map(|a| { let b: f64 = a.clone().into(); b }).collect()
    }
}

#[derive(Deserialize)]
struct Materia {
    pub grades: Vec<MateriaTier>,
}

#[derive(Deserialize)]
struct MateriaTier {
    pub name: String,
    pub materia: HashMap<String, u32>,
}