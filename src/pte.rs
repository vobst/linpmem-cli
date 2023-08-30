use anyhow::bail;
use std::fmt::{self, Display, Formatter};
use clap::ValueEnum;

pub struct Pte {
    pub value: u64,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum PteParts {
    Xd,
    Pk,
    Avl1,
    Pfn,
    Avl2,
    G,
    Pat,
    D,
    A,
    Pcd,
    Pwt,
    Us,
    Rw,
    P,
}

impl PteParts {
    fn shift(&self) -> u64 {
        match self {
            &Self::Xd => (63),
            &Self::Pk => (59),
            &Self::Avl1 => (52),
            &Self::Pfn => (12),
            &Self::Avl2 => (9),
            &Self::G => (8),
            &Self::Pat => (7),
            &Self::D => (6),
            &Self::A => (5),
            &Self::Pcd => (4),
            &Self::Pwt => (3),
            &Self::Us => (2),
            &Self::Rw => (1),
            &Self::P => (0),
        }
    }

    fn mask(&self) -> u64 {
        match self {
            &Self::Xd => ((1 as u64) << 63),
            &Self::Pk => ((0xF as u64) << 59),
            &Self::Avl1 => ((0x7F as u64) << 52),
            &Self::Pfn => ((0xFFFFFFFFFF as u64) << 12),
            &Self::Avl2 => ((3 as u64) << 9),
            &Self::G => ((1 as u64) << 8),
            &Self::Pat => ((1 as u64) << 7),
            &Self::D => ((1 as u64) << 6),
            &Self::A => ((1 as u64) << 5),
            &Self::Pcd => ((1 as u64) << 4),
            &Self::Pwt => ((1 as u64) << 3),
            &Self::Us => ((1 as u64) << 2),
            &Self::Rw => ((1 as u64) << 1),
            &Self::P => ((1 as u64) << 0),
        }
    }
}

impl TryFrom<&str> for PteParts
{
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<PteParts> {
        Ok(match value.as_ref() {
            "xd" => PteParts::Xd,
            "pk" => PteParts::Pk,
            "avl1" => PteParts::Avl1,
            "pfn" => PteParts::Pfn,
            "avl2" =>PteParts::Avl2,
            x => bail!("Cannot parse {} as PTE part", x),
        })
    }
}

impl From<u64> for Pte {
    fn from(value: u64) -> Self {
        Self {
            value
        }
    }
}

impl Display for Pte {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result{
        println!("0x{:016x}", self.value);
        Ok(())
    }
}


impl TryFrom<&Vec<PteParts>> for Pte {
    type Error = anyhow::Error;

    fn try_from(value: &Vec<PteParts>) -> anyhow::Result<Self> {
        let mut pte = Pte::new();

        for part in value {
            pte = match part {
                PteParts::Xd => pte.xd(),
                PteParts::G => pte.g(),
                PteParts::Pat => pte.pat(),
                PteParts::D => pte.d(),
                PteParts::A => pte.a(),
                PteParts::Pcd => pte.pcd(),
                PteParts::Pwt => pte.pwt(),
                PteParts::Us => pte.us(),
                PteParts::Rw => pte.rw(),
                PteParts::P => pte.p(),
                _ => bail!(
                    "Only single bit fields are allowed when assembling PTEs from parts"
                ),
            }
        }

        Ok(pte)
    }
}

impl Pte {
    fn new() -> Self {
        Self { value: 0 }
    }

    fn xd(mut self) -> Self {
        self.value |= PteParts::Xd.mask();

        self
    }

    fn pk(mut self, value: u64) -> Self {
        self.value |= ((value << PteParts::Pk.shift()) & PteParts::Pk.mask());

        self
    }

    fn pfn(mut self, value: u64) -> Self {
        self.value |= ((value << PteParts::Pfn.shift()) & PteParts::Pfn.mask());

        self
    }

    fn g(mut self) -> Self {
        self.value |= PteParts::G.mask();

        self
    }

    fn pat(mut self) -> Self {
        self.value |= PteParts::Pat.mask();

        self
    }

    fn d(mut self) -> Self {
        self.value |= PteParts::D.mask();

        self
    }

    fn a(mut self) -> Self {
        self.value |= PteParts::A.mask();

        self
    }

    fn pcd(mut self) -> Self {
        self.value |= PteParts::Pcd.mask();

        self
    }

    fn pwt(mut self) -> Self {
        self.value |= PteParts::Pwt.mask();

        self
    }

    fn us(mut self) -> Self {
        self.value |= PteParts::Us.mask();

        self
    }

    fn rw(mut self) -> Self {
        self.value |= PteParts::Rw.mask();

        self
    }

    fn p(mut self) -> Self {
        self.value |= PteParts::P.mask();

        self
    }
}
