// src/poseidon_bn254.rs

use num_bigint::BigUint;
use num_traits::Num;
use std::borrow::Borrow;

use ark_bn254::Fr;
use ark_crypto_primitives::{
    crh::poseidon::constraints::CRHParametersVar,
    sponge::{
        constraints::CryptographicSpongeVar,
        poseidon::{constraints::PoseidonSpongeVar, PoseidonConfig, PoseidonSponge},
        CryptographicSponge,
    },
};
use ark_r1cs_std::{
    alloc::{AllocVar, AllocationMode},
    fields::fp::FpVar,
    R1CSVar,
};
use ark_relations::r1cs::{Namespace, SynthesisError};

pub fn poseidon_bn254() -> PoseidonConfig<Fr> {
    let round_constants = [
        "4417881134626180770308697923359573201005643519861877412381846989312604493735",
        "5433650512959517612316327474713065966758808864213826738576266661723522780033",
        "16762637755406472493812601151014278118641635553391913542964880219707263322052",
        "17301668213014761646596552653015591039760461391617952799153519234591864034463",
        "19776162473771124504636460540005465934623588388516917789964545078054814040751",
        "11316768622745143833150712249439306411966930091146927416794626237074092972974",
        "16445483823163065295596987123801926703922762982696126295152762759513382273866",
        "4514474156763384993737907502541146539969481658202448264361041161669007485071",
        "15962945726901666037200703277090943625692830090147479512076829448913210994056",
        "887718591790650017281197227986729839639624303500401204697133087008458682956",
        "12274216425815286338344348482336276117995066724093487299512262174649563976186",
        "1050758930252644049914605206403427631313247657935800041327915318465466974783",
        "10607585076226348745183629788245008577438579576330359487986117688517235408881",
        "20509009694313778489858884111978287368685508463213168251197030962719682409205",
        "4563680725198793251172562411623420668869330885202539013185566348548255071635",
        "10132793020925051358967312895903080509340240497490871556356344077873835967889",
        "2647212931513679432767054030363504863222540192567519779190979594289737202550",
        "4359886051856780224292971980741406425492649906224296730054335077893394909530",
        "3054847578866604975257033821021111653343035155943751913629683293137065347688",
        "1596959724864208462318973909411993812742702283705404297188991879075781032285",
        "9261998448432672939016143689737142959950360294646800435636112732010430895748",
        "21811458719499960186771214587366397959723618323848154217660010569954685810715",
        "3239076967784329489572444293576130919094218653342995300018245457598795548917",
        "20462520400712402627709344205131902699657125916001693532154815488581362749475",
        "15540512546148021086829191867945740637058588949955294856096961322685397103557",
        "13865340336144010740428645313739899438060242120535852433410522037321843803156",
        "4030925228209360989389332579897661669475428247758360576989046703985980534507",
        "2580022244547148022781785697172503311467595214847728559144402326933969076319",
        "337091914144357888262743136529662076143869414926179844835581507288841653842",
        "21128818615001540137263856211467089199630793116539340697922423416110745118506",
        "7314289664170998780822250984795805793842874757314808790503348444042779760074",
        "17774049587694351616089815110956846453850106707275621110789201687277957820281",
        "11019693257420674975398059669532837767980992812803956883547482631461450651960",
        "8945986251101329707360666524341408357502651307257925367450947820355368214535",
        "16600645183477638722007904287733521074791737381945794958827103590279914462576",
        "11176512602209691417636272193861235132071391793840565562361651389294061042059",
        "14889617408341048364135563212181036206522304313837983213620424531022449761460",
        "13725846858893036951158965143583678272726150787955443316044240114688865768260",
        "7031966701933394996335226463904734684231938370740242159297465282149711323302",
        "1598623817079294552053879422210947472173483306271647853752616124799192256195",
        "19774466718056564032734488707833648551011540998602523750214654491716642104404",
        "10833489778171075947445720806211760058004501131759886332251918804308190690081",
        "1643230281589922929913461313269721677840560726699527243824368999070606462076",
        "5529383825244679302765929967407905660423153508826597450732592666784449667225",
        "3182718166498008812418586185198779390426386793456672324496083987133297273028",
        "14599400764263231688213618032482449522109778447311452342990557992771279312515",
        "19971122460007657859508211404523175040055308424338161607196356611508685672711",
        "12801369279155577131760037436686619341421686678830792595575775840669776442563",
        "2874800145470695258580041840911551878272283565060591475417515522159069870291",
        "15558578381003392888309173936039806021949114680855373870727168432126175091041",
        "9811599964264530187304305794911397412166832842489170888364387877850235210426",
        "11797479380255190086030457967941350172480879235135372778366002137304980472711",
        "15812276861397201227532067085271584728094218111982315967331671382133625425702",
        "14555033526911765831054005951477598685743695466168667850533171876658643729676",
        "8453370176619636688730899865196838058367647719211769227158095525122045566926",
        "13746050345785052659407845574049356403121452981555493239274450074369130425982",
        "14759221361221742392877550517283338785302973165056606116918148040408389214809",
        "12918384943133505870596332126482017188382093626741625148832869110825500058603",
        "6090669736471884927589246515939581624927624112611999595504716515722644260809",
        "5818256914990297609452278687275570020517086210519703828397962481130049333349",
        "21665668991006174860786913595488512200171275130423190085616080866607498513357",
        "2268297495968854614780848291068505574765456615293615512346180903730018964697",
        "13973211016421000871032597822807506910708863418864348758128380360422656258892",
        "11877184595954796005081407550764117823731241224470762866371883626576306508793",
        "2986065710695845701959971763474802354333410127136499986005712979427956719303",
        "12649794127562509279197585900369868076945168057811626745015439261348449322516",
        "12275810531539430738053742281045622394839331044128025660443842052759513336728",
        "21143091624051898429049779202777783090616631078773674742631481136382890245166",
        "10171452642243387955781526332645193123346266850900972162093561209595357586423",
        "3855895713134056811998624598511873826580243324673003091901619068671473231435",
        "13528768565745931233460851026527843568952293200662922221190432010518079703355",
        "6443318563434187156482979037576690662213493094314678699384407936675395356263",
    ];

    let mds = [
        [
            "2910766817845651019878574839501801340070030115151021261302834310722729507541",
            "14876694094903316616163091687595355836267453073383265044550370713659048938454",
        ],
        [
            "19727366863391167538122140361473584127147630672623100827934084310230022599144",
            "7527312705817953459920138003796377030820958175883853967715612380516078993222",
        ],
    ];

    let ark: Vec<Vec<Fr>> = round_constants
        .iter()
        .map(|e| {
            vec![Fr::from(
                BigUint::from_str_radix(e, 10).expect("Failed to parse round constant"),
            )]
        })
        .collect::<Vec<_>>();

    let mds = mds
        .map(|row| {
            row.map(|e| {
                Fr::from(
                    BigUint::from_str_radix(e, 10).expect("Failed to parse MDS matrix element"),
                )
            })
            .to_vec()
        })
        .to_vec();

    PoseidonConfig::<Fr> {
        full_rounds: 8,
        partial_rounds: 56,
        alpha: 5,
        ark,
        mds,
        rate: 2,
        capacity: 1,
    }
}

/// Native Poseidon hash (BN254) with 1-, 2- and 3-input helpers.
#[derive(Debug, Clone)]
pub struct PoseidonHash {
    pub config: PoseidonConfig<Fr>,
}

impl PoseidonHash {
    pub fn new(config: PoseidonConfig<Fr>) -> Self {
        Self { config }
    }

    pub fn hash1(&self, x: &Fr) -> Fr {
        let mut sponge = PoseidonSponge::new(&self.config);
        sponge.absorb(x);
        let out = sponge.squeeze_field_elements::<Fr>(1);
        out[0]
    }

    pub fn hash2(&self, left: &Fr, right: &Fr) -> Fr {
        let mut sponge = PoseidonSponge::new(&self.config);
        sponge.absorb(left);
        sponge.absorb(right);
        let out = sponge.squeeze_field_elements::<Fr>(1);
        out[0]
    }

    pub fn hash3(&self, a: &Fr, b: &Fr, c: &Fr) -> Fr {
        let mut sponge = PoseidonSponge::new(&self.config);
        sponge.absorb(a);
        sponge.absorb(b);
        sponge.absorb(c);
        let out = sponge.squeeze_field_elements::<Fr>(1);
        out[0]
    }

    /// Hash an array/slice of field elements into a single field element
    pub fn hash_array(&self, elements: &[Fr]) -> Fr {
        let mut sponge = PoseidonSponge::new(&self.config);
        for elem in elements {
            sponge.absorb(elem);
        }
        let out = sponge.squeeze_field_elements::<Fr>(1);
        out[0]
    }
}

/// Constraint gadget for Poseidon hash (BN254) with 1-, 2-, and 3-input helpers.
pub struct PoseidonHashVar {
    pub config: CRHParametersVar<Fr>,
}

impl PoseidonHashVar {
    pub fn hash1(&self, x: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
        let cs = x.cs();

        // All constants: compute natively and return a constant var.
        if cs.is_none() {
            let nx = x.value()?;
            let native = PoseidonHash::new(self.config.parameters.clone()).hash1(&nx);
            return Ok(FpVar::Constant(native));
        }

        // At least one witness: use sponge gadget.
        let mut sponge = PoseidonSpongeVar::new(cs, &self.config.parameters);
        sponge.absorb(x)?;
        let out = sponge.squeeze_field_elements(1)?;
        Ok(out[0].clone())
    }

    pub fn hash2(&self, left: &FpVar<Fr>, right: &FpVar<Fr>) -> Result<FpVar<Fr>, SynthesisError> {
        let cs = left.cs().or(right.cs());

        if cs.is_none() {
            let native_left = left.value()?;
            let native_right = right.value()?;

            let native = PoseidonHash::new(self.config.parameters.clone())
                .hash2(&native_left, &native_right);

            return Ok(FpVar::Constant(native));
        }

        let mut sponge = PoseidonSpongeVar::new(cs, &self.config.parameters);
        sponge.absorb(left)?;
        sponge.absorb(right)?;
        let out = sponge.squeeze_field_elements(1)?;
        Ok(out[0].clone())
    }

    pub fn hash3(
        &self,
        a: &FpVar<Fr>,
        b: &FpVar<Fr>,
        c: &FpVar<Fr>,
    ) -> Result<FpVar<Fr>, SynthesisError> {
        let cs = a.cs().or(b.cs()).or(c.cs());

        if cs.is_none() {
            let na = a.value()?;
            let nb = b.value()?;
            let nc = c.value()?;

            let native = PoseidonHash::new(self.config.parameters.clone()).hash3(&na, &nb, &nc);

            return Ok(FpVar::Constant(native));
        }

        let mut sponge = PoseidonSpongeVar::new(cs, &self.config.parameters);
        sponge.absorb(a)?;
        sponge.absorb(b)?;
        sponge.absorb(c)?;
        let out = sponge.squeeze_field_elements(1)?;
        Ok(out[0].clone())
    }

    /// Hash an array/slice of field elements into a single field element
    pub fn hash_array(&self, elements: &[FpVar<Fr>]) -> Result<FpVar<Fr>, SynthesisError> {
        if elements.is_empty() {
            return Err(SynthesisError::AssignmentMissing);
        }

        // Get constraint system from first element (like hash1 does with x.cs())
        let cs = elements[0].cs();

        // If all are constants, compute natively
        if cs.is_none() {
            let native_elements: Vec<Fr> = elements
                .iter()
                .map(|e| e.value())
                .collect::<Result<Vec<_>, _>>()?;

            let native =
                PoseidonHash::new(self.config.parameters.clone()).hash_array(&native_elements);
            return Ok(FpVar::Constant(native));
        }

        // At least one witness: use sponge gadget
        let mut sponge = PoseidonSpongeVar::new(cs, &self.config.parameters);
        for elem in elements {
            sponge.absorb(elem)?;
        }
        let out = sponge.squeeze_field_elements(1)?;
        Ok(out[0].clone())
    }
}

/// Allocate PoseidonHashVar from a PoseidonHash (native wrapper).
impl AllocVar<PoseidonHash, Fr> for PoseidonHashVar {
    fn new_variable<T: Borrow<PoseidonHash>>(
        cs: impl Into<Namespace<Fr>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        f().and_then(|param| {
            let parameters = param.borrow();
            let cfg_var = CRHParametersVar::new_variable(cs, || Ok(&parameters.config), mode)?;
            Ok(Self { config: cfg_var })
        })
    }
}

/// Allocate PoseidonHashVar directly from a PoseidonConfig<Fr>.
impl AllocVar<PoseidonConfig<Fr>, Fr> for PoseidonHashVar {
    fn new_variable<T: Borrow<PoseidonConfig<Fr>>>(
        cs: impl Into<Namespace<Fr>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: AllocationMode,
    ) -> Result<Self, SynthesisError> {
        f().and_then(|param| {
            let cfg = param.borrow();
            let cfg_var = CRHParametersVar::new_variable(cs, || Ok(cfg), mode)?;
            Ok(Self { config: cfg_var })
        })
    }
}
