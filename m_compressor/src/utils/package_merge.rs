#[derive(Clone, Copy)]
enum ItemClass {
    Leaf(usize),
    Package(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_limited_code_lengths_empty() {
        let freqs: Vec<usize> = vec![];
        let lengths = get_limited_code_lengths(&freqs, 15);
        assert!(lengths.is_empty());
    }

    #[test]
    fn test_get_limited_code_lengths_single_symbol() {
        let freqs = vec![10];
        let lengths = get_limited_code_lengths(&freqs, 15);
        assert_eq!(lengths.len(), 1);
        assert_eq!(lengths[0], 1);
    }

    #[test]
    fn test_get_limited_code_lengths_two_symbols() {
        let freqs = vec![5, 10];
        let lengths = get_limited_code_lengths(&freqs, 15);
        assert_eq!(lengths.len(), 2);
        assert!(lengths[0] > 0);
        assert!(lengths[1] > 0);
    }

    #[test]
    fn test_get_limited_code_lengths_with_zeros() {
        let freqs = vec![0, 10, 0, 5, 0];
        let lengths = get_limited_code_lengths(&freqs, 15);
        assert_eq!(lengths.len(), 5);
        assert_eq!(lengths[0], 0);
        assert_eq!(lengths[2], 0);
        assert_eq!(lengths[4], 0);
        assert!(lengths[1] > 0);
        assert!(lengths[3] > 0);
    }

    #[test]
    fn test_get_limited_code_lengths_all_zeros() {
        let freqs = vec![0, 0, 0, 0, 0];
        let lengths = get_limited_code_lengths(&freqs, 15);
        assert_eq!(lengths.len(), 5);
        assert!(lengths.iter().all(|&len| len == 0));
    }

    #[test]
    fn test_get_limited_code_lengths_max_length() {
        let freqs = vec![1, 2, 3, 4, 5];
        let max_len = 15;
        let lengths = get_limited_code_lengths(&freqs, max_len);
        assert_eq!(lengths.len(), 5);
        assert!(lengths.iter().all(|&len| len <= max_len));
    }

    #[test]
    fn test_get_limited_code_lengths_small_max() {
        let freqs = vec![1, 1, 1, 1, 1];
        let max_len = 3;
        let lengths = get_limited_code_lengths(&freqs, max_len);
        assert_eq!(lengths.len(), 5);
        assert!(lengths.iter().all(|&len| len <= max_len));
    }

    #[test]
    fn test_get_limited_code_lengths_codebook_property() {
        let freqs = vec![10, 20, 30, 40];
        let lengths = get_limited_code_lengths(&freqs, 15);

        let max_len = *lengths.iter().max().unwrap() as usize;
        let mut counts = vec![0; max_len + 1];
        for &len in &lengths {
            if len > 0 {
                counts[len as usize] += 1;
            }
        }

        for l in 1..=max_len {
            if counts[l] > 1 {
                let mut sum = 0;
                for j in l..=max_len {
                    sum += counts[j] << (j - l);
                }
                assert!(sum <= 2, "Codebook property violated at length {}", l);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Item {
    weight: usize,
    class: ItemClass,
}

fn unpack(levels: &Vec<Vec<Item>>, beg_lev_ind: usize, beg_item_ind: usize, lengths: &mut Vec<u8>) {
    let mut stack: Vec<(usize, usize)> = Vec::new();
    stack.push((beg_lev_ind, beg_item_ind));

    while !stack.is_empty() {
        let (lev_ind, item_ind) = stack.pop().unwrap();

        match levels[lev_ind][item_ind].class {
            ItemClass::Leaf(sym) => {
                lengths[sym] += 1;
            }
            ItemClass::Package(pre_ind_1, pre_ind_2) => {
                stack.push((lev_ind - 1, pre_ind_2));
                stack.push((lev_ind - 1, pre_ind_1));
            }
        }
    }
}

pub fn get_limited_code_lengths(freqs: &[usize], max_bit_len: u8) -> Vec<u8> {
    let symbols: Vec<usize> = freqs
        .iter()
        .enumerate()
        .filter(|&(_, &f)| f > 0)
        .map(|(i, _)| i)
        .collect();
    let mut res_lengths = vec![0u8; freqs.len()];

    if symbols.is_empty() {
        return res_lengths;
    }

    if symbols.len() == 1 {
        res_lengths[symbols[0]] = 1;
        return res_lengths;
    }

    let mut base_level: Vec<Item> = symbols
        .iter()
        .map(|&s| -> Item {
            return Item {
                weight: freqs[s],
                class: ItemClass::Leaf(s),
            };
        })
        .collect();

    base_level.sort_by_key(|item| item.weight);

    let mut levels: Vec<Vec<Item>> = Vec::with_capacity(max_bit_len as usize);
    levels.push(base_level);

    for cur_ind in 1..max_bit_len as usize {
        let mut current_level = Vec::new();
        let prev_level = &levels[cur_ind - 1];
        let mut packages = Vec::with_capacity(prev_level.len() / 2);

        for i in (0..prev_level.len()).step_by(2) {
            if i + 1 < prev_level.len() {
                packages.push(Item {
                    weight: prev_level[i].weight + prev_level[i + 1].weight,
                    class: ItemClass::Package(i, i + 1),
                });
            }
        }

        let base_level_ref = &levels[0];
        let mut leaf_ind = 0;
        let mut pkg_ind = 0;

        while leaf_ind < base_level_ref.len() || pkg_ind < packages.len() {
            if leaf_ind < base_level_ref.len()
                && (pkg_ind == packages.len()
                    || base_level_ref[leaf_ind].weight <= packages[pkg_ind].weight)
            {
                current_level.push(base_level_ref[leaf_ind]);
                leaf_ind += 1;
            } else {
                current_level.push(packages[pkg_ind]);
                pkg_ind += 1;
            }
        }
        levels.push(current_level);
    }

    let last_level_ind = max_bit_len as usize - 1;
    let last_level = &levels[last_level_ind];
    let num_to_pick = (symbols.len() - 1) << 1;

    for ind in 0..num_to_pick.min(last_level.len()) {
        unpack(&levels, last_level_ind, ind, &mut res_lengths);
    }

    res_lengths
}
