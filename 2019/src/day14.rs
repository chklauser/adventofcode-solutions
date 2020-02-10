use std::rc::Rc;
use fxhash::FxHashMap;

#[aoc_generator(day14)]
pub fn generator(input: &str) -> Vec<Recipe> {
    let mut elements: FxHashMap<&str, Rc<String>> = FxHashMap::default();
    fn intern<'a>(elements: &mut FxHashMap<&'a str, Rc<String>>, id: &'a str) -> Rc<String> {
        elements.entry(id).or_insert_with(|| Rc::new(id.to_owned())).clone()
    }
    fn parse_ingredient<'a>(elements: &mut FxHashMap<&'a str, Rc<String>>, raw: &'a str) -> Ingredient {
        let t: (isize, &'_ str) = serde_scan::from_str(raw.trim()).expect("AMOUNT INGREDIENT");
        Ingredient {
            amount: t.0,
            element: intern(elements, t.1)
        }
    }
    input.lines().map(|line| {
        let mut parts = line.splitn(2, "=>");
        let lhs : &'_ str = parts.next().expect("recipe must have lhs");
        let rhs : &'_ str = parts.next().expect("recipe must have rhs").trim();
        let ingredient_parts = lhs.split(',');
        let ingredients = ingredient_parts
            .map(|p| parse_ingredient(&mut elements, p))
            .collect();
        Recipe {
            input: ingredients,
            output: parse_ingredient(&mut elements, rhs)
        }
    }).collect()
}

#[derive(Debug,Eq,PartialEq,Clone)]
pub struct Recipe{
    input: Vec<Ingredient>,
    output: Ingredient
}

#[derive(Debug,Eq,PartialEq,Clone)]
pub struct Ingredient {
    element: Rc<String>,
    amount: isize
}
impl Ingredient {
    fn times(&self, multiplier: isize) -> Ingredient {
        Ingredient {
            element: self.element.clone(),
            amount: self.amount * multiplier
        }
    }
}

#[aoc(day14, part1)]
pub fn part1(input: &Vec<Recipe>) -> isize {
    let recipies: FxHashMap<Rc<String>, &Recipe> = input.into_iter().map(|r| (r.output.element.clone(), r)).collect();
    let mut needs : FxHashMap<Rc<String>, isize> = FxHashMap::default();

    needs.insert(Rc::new("FUEL".to_owned()), 1);
    satisfy_needs(&recipies, &mut needs, &mut Vec::new());
    *needs.get(&"ORE".to_owned()).expect("ORE needs")
}

#[aoc(day14, part2)]
pub fn part2(input: &Vec<Recipe>) -> isize {
    eprintln!("input: {:?}", input);
    let recipies: FxHashMap<Rc<String>, &Recipe> = input.into_iter().map(|r| (r.output.element.clone(), r)).collect();
    let mut needs : FxHashMap<Rc<String>, isize> = FxHashMap::default();

    let fuel_id = recipies.get_key_value(&"FUEL".to_owned()).expect("FUEL to be a recipe").0.clone();
    let ore_id = Rc::new("ORE".to_owned());

    needs.insert(ore_id.clone(), -1_000_000_000_000);
    fn need_more_fuel(needs: &mut FxHashMap<Rc<String>, isize>, fuel_id: &Rc<String>) {
        let required_amount = needs.entry(fuel_id.clone()).or_insert(0);
        *required_amount += 1;
    }
    eprintln!("needs: {:?}", needs);

    let mut fuel_produced = 0isize;
    let mut buf = Vec::new();
    while *needs.get(&ore_id).expect("we just inserted ore") < 0 {
        need_more_fuel(&mut needs, &fuel_id);
        satisfy_needs(&recipies, &mut needs, &mut buf);
        fuel_produced += 1;
        if fuel_produced % 50000 == 0 {
            let ore_left = -*needs.get(&ore_id).expect("we just inserted ore");
            eprintln!("fuel: {}, ore: {}, pct: {:.2}%", fuel_produced, ore_left, ore_left as f64/1_000_000_000_000.0*100.0);
        }
    }

    let final_ore_need = *needs.get(&ore_id).expect("ORE needs");
    if final_ore_need > 0 {
        fuel_produced - 1
    } else {
        fuel_produced
    }
}

fn satisfy_needs(recipies: &FxHashMap<Rc<String>, &Recipe>, needs: &mut FxHashMap<Rc<String>, isize>, buf: &mut Vec<Ingredient>) -> () {
    fn num_applications(mut amount_needed: isize, recipe_output: isize) -> isize {
        let mut num = 0isize;
        while amount_needed > 0 {
            num += 1;
            amount_needed -= recipe_output;
        }
        num
    }
    fn need_to_continue(needs: &FxHashMap<Rc<String>, isize>) -> bool {
        needs.iter().any(|n| &n.0[..] != "ORE" && *n.1 > 0)
    }

    buf.clear();
    while need_to_continue(&*needs) {
        //eprintln!("needs: {:?}", needs);
        for (elem, amount_needed) in &*needs {
            if &elem[..] == "ORE" || *amount_needed <= 0 {
                continue;
            }
            let recipe = *recipies.get(elem).expect("missing recipe");
            let num = num_applications(*amount_needed, recipe.output.amount);
            //eprintln!("fabbing {} units of {}", num*recipe.output.amount, recipe.output.element);
            buf.extend(recipe.input.iter().map(|i| i.times(num)));
            buf.push(recipe.output.times(-num));
        }
        for change in &*buf {
            let required_amount = needs.entry(change.element.clone()).or_insert(0);
            *required_amount += change.amount;
        }
        buf.clear();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn ingredient(amount: isize, element: &str) -> Ingredient {
        Ingredient {
            amount,
            element: Rc::new(element.to_owned())
        }
    }

    #[test]
    fn mini_recipe() {
        assert_eq!(generator("1 A=>2 B"), vec![Recipe {
            input: vec![ingredient(1, "A")],
            output: ingredient(2, "B")
        }]);
    }

    #[test]
    fn space_recipe() {
        assert_eq!(generator("      1 A    =>    2    B   "), vec![Recipe {
            input: vec![ingredient(1, "A")],
            output: ingredient(2, "B")
        }]);
    }

    #[test]
    fn large_recipe() {
        assert_eq!(generator("2 LCHZ, 13 JTJT, 10 TPXCK => 3 RSZF"), vec![Recipe{
            input: vec![ingredient(2, "LCHZ"), ingredient(13, "JTJT"), ingredient(10, "TPXCK")],
            output: ingredient(3, "RSZF")
        }]);
    }

    #[test]
    fn multiple_recipes() {
        assert_eq!(generator("1 A=>2 B\n2 C => 4 D"), vec![
            Recipe {
                input: vec![ingredient(1, "A")],
                output: ingredient(2, "B")
            },
            Recipe {
                input: vec![ingredient(2, "C")],
                output: ingredient(4, "D")
            }
        ]);
    }

    #[test]
    fn part1_example1() {
        assert_eq!(part1(&generator("10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL")), 31);
    }

    #[test]
    fn part1_example2() {
        assert_eq!(part1(&generator("9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL")), 165);
    }

    #[test]
    fn part1_example3() {
        assert_eq!(part1(&generator("157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT")), 13312);
    }

    #[test]
    fn part1_example4() {
        assert_eq!(part1(&generator("2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF")), 180697);
    }

    #[test]
    fn part1_example5() {
        assert_eq!(part1(&generator("171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX")), 2210736);
    }

    // the part2 tests are mega-slow (take minutes to complete)

    #[test]
    fn part2_example3() {
        assert_eq!(part2(&generator("157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT")), 82892753);
    }



    #[test]
    fn part2_example4() {
        assert_eq!(part2(&generator("2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF")), 5586022);
    }

    #[test]
    fn part2_example5() {
        assert_eq!(part2(&generator("171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX")), 460664 );
    }
}