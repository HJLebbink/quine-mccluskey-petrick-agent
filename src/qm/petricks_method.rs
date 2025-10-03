use super::encoding::MintermEncoding;
use super::implicant::Implicant;

pub struct PetricksMethod<E: MintermEncoding> {
    prime_implicants: Vec<Implicant<E>>,
    minterms: Vec<E::Value>,
}

impl<E: MintermEncoding> PetricksMethod<E> {
    pub fn new(prime_implicants: &[Implicant<E>], minterms: &[E::Value]) -> Self {
        Self {
            prime_implicants: prime_implicants.to_vec(),
            minterms: minterms.to_vec(),
        }
    }

    pub fn find_minimal_cover(&self) -> Vec<Implicant<E>> {
        if self.prime_implicants.is_empty() {
            return Vec::new();
        }

        let mut covered_minterms = std::collections::HashSet::new();
        let mut selected = Vec::new();

        for pi in &self.prime_implicants {
            let mut covers_new = false;
            for &minterm in &self.minterms {
                if pi.covers_minterm(minterm) && !covered_minterms.contains(&minterm) {
                    covers_new = true;
                    break;
                }
            }

            if covers_new {
                selected.push(pi.clone());
                for &minterm in &pi.covered_minterms {
                    covered_minterms.insert(minterm);
                }
            }

            if covered_minterms.len() >= self.minterms.len() {
                break;
            }
        }

        selected
    }

    pub fn generate_product_of_sums(&self) -> String {
        "Dummy POS expression".to_string()
    }
}