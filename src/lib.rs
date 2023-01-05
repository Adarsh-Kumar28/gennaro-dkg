//! Gennaro's Distributed Key Generation Algorithm.
//!
//! The algorithm uses participants with unique identifiers
//! and each party communicates broadcast data and peer-to-peer
//! data depending on the round. Round 1 generates participant key shares
//! which are checked for correctness in round 2. Any participant that fails
//! in round 2 is dropped from the valid set which is communicated in round 3.
//! Round 4 communicates only with the remaining valid participants
//! and computes the secret share and verification key. Round 5 checks that
//! all participants computed the same verification key.
//!
//! The idea is that Rounds 3 and 5 serve as echo broadcasts to check the
//! state of all valid participants. If an error occurs in any round, then
//! participants either drop invalid participants or abort.
//!
//! The full paper can be found
//! <https://link.springer.com/content/pdf/10.1007/s00145-006-0347-3.pdf>.
//!
//! The interface has been written to work with anything that implements the elliptic-curve::Group
//! trait.
//!
//! An example for generating a secret key on the Secp256k1 curve with 2 out of 3 participants.
//!
//! ```
//! use elliptic_curve::{Group, PrimeField};
//! use gennaro_dkg::*;
//! use k256::{ProjectivePoint, Scalar};
//! use maplit::btreemap;
//! use std::{
//!     collections::BTreeMap,
//!     num::NonZeroUsize,
//! };
//! use vsss_rs::{Shamir, Share};
//!
//! let parameters = Parameters::new(NonZeroUsize::new(2).unwrap(), NonZeroUsize::new(3).unwrap());
//!
//! let mut participant1 = Participant::<ProjectivePoint>::new(NonZeroUsize::new(1).unwrap(), parameters).unwrap();
//! let mut participant2 = Participant::<ProjectivePoint>::new(NonZeroUsize::new(2).unwrap(), parameters).unwrap();
//! let mut participant3 = Participant::<ProjectivePoint>::new(NonZeroUsize::new(3).unwrap(), parameters).unwrap();
//!
//! // Round 1
//! let (b1data1, p2p1data) = participant1.round1().unwrap();
//! let (b2data1, p2p2data) = participant2.round1().unwrap();
//! let (b3data1, p2p3data) = participant3.round1().unwrap();
//!
//! // Can't call the same round twice
//! assert!(participant1.round1().is_err());
//! assert!(participant2.round1().is_err());
//! assert!(participant3.round1().is_err());
//!
//! // Send b1data1 to participant 2 and 3
//! // Send b2data1 to participant 1 and 3
//! // Send b3data1 to participant 1 and 2
//!
//! // Send p2p1data[&2] to participant 2
//! // Send p2p1data[&3] to participant 3
//!
//! // Send p2p2data[&1] to participant 1
//! // Send p2p2data[&3] to participant 3
//!
//! // Send p2p3data[&1] to participant 1
//! // Send p2p3data[&2] to participant 2
//!
//! let p1bdata1 = btreemap! {
//!     2 => b2data1.clone(),
//!     3 => b3data1.clone(),
//! };
//! let p1pdata = btreemap! {
//!     2 => p2p2data[&1].clone(),
//!     3 => p2p3data[&1].clone(),
//! };
//! let b1data2 = participant1.round2(p1bdata1, p1pdata).unwrap();
//!
//! let p2bdata1 = btreemap! {
//!     1 => b1data1.clone(),
//!     3 => b3data1.clone(),
//! };
//! let p2pdata = btreemap! {
//!     1 => p2p1data[&2].clone(),
//!     3 => p2p3data[&2].clone(),
//! };
//! let b2data2 = participant2.round2(p2bdata1, p2pdata).unwrap();
//!
//! let p3bdata1 = btreemap! {
//!     1 => b1data1.clone(),
//!     2 => b2data1.clone(),
//! };
//! let p3pdata = btreemap! {
//!     1 => p2p1data[&3].clone(),
//!     2 => p2p2data[&3].clone(),
//! };
//! let b3data2 = participant3.round2(p3bdata1, p3pdata).unwrap();
//!
//! // Send b1data2 to participants 2 and 3
//! // Send b2data2 to participants 1 and 3
//! // Send b3data2 to participants 1 and 2
//!
//! // This is an optimization for the example in reality each participant computes this
//! let bdata2 = btreemap! {
//!     1 => b1data2,
//!     2 => b2data2,
//!     3 => b3data2,
//! };
//!
//! let b1data3 = participant1.round3(&bdata2).unwrap();
//! let b2data3 = participant2.round3(&bdata2).unwrap();
//! let b3data3 = participant3.round3(&bdata2).unwrap();
//!
//! // Send b1data3 to participants 2 and 3
//! // Send b2data3 to participants 1 and 3
//! // Send b3data3 to participants 1 and 2
//!
//! // This is an optimization for the example in reality each participant computes this
//! let bdata3 = btreemap! {
//!     1 => b1data3,
//!     2 => b2data3,
//!     3 => b3data3,
//! };
//!
//! let b1data4 = participant1.round4(&bdata3).unwrap();
//! let b2data4 = participant2.round4(&bdata3).unwrap();
//! let b3data4 = participant3.round4(&bdata3).unwrap();
//!
//! // Send b1data4 to participants 2 and 3
//! // Send b2data4 to participants 1 and 3
//! // Send b3data4 to participants 1 and 2
//!
//! // Verify that the same key is computed then done
//!
//! // This is an optimization for the example in reality each participant computes this
//! let bdata4 = btreemap! {
//!     1 => b1data4,
//!     2 => b2data4,
//!     3 => b3data4,
//! };
//!
//! assert!(participant1.round5(&bdata4).is_ok());
//! assert!(participant2.round5(&bdata4).is_ok());
//! assert!(participant3.round5(&bdata4).is_ok());
//!
//! // Get the verification key
//! let pk1 = participant1.get_public_key();
//! // Get the secret share
//! let share1 = participant1.get_secret_share();
//!
//! assert_eq!(pk1.is_identity().unwrap_u8(), 0u8);
//! assert_eq!(share1.is_zero().unwrap_u8(), 0u8);
//!
//! let pk2 = participant2.get_public_key();
//! let share2 = participant2.get_secret_share();
//!
//! assert_eq!(pk2.is_identity().unwrap_u8(), 0u8);
//! assert_eq!(share2.is_zero().unwrap_u8(), 0u8);
//!
//! let pk3 = participant3.get_public_key();
//! let share3 = participant3.get_secret_share();
//!
//! assert_eq!(pk3.is_identity().unwrap_u8(), 0u8);
//! assert_eq!(share3.is_zero().unwrap_u8(), 0u8);
//!
//! // Public keys will be equal
//! assert_eq!(pk1, pk2);
//! assert_eq!(pk2, pk3);
//! // Shares should not be
//! assert_ne!(share1, share2);
//! assert_ne!(share1, share3);
//! assert_ne!(share2, share3);
//!
//! // For demonstration purposes, the shares if collected can be combined to recreate
//! // the computed secret
//!
//! let mut s1 = share1.to_repr().to_vec();
//! let mut s2 = share2.to_repr().to_vec();
//! let mut s3 = share3.to_repr().to_vec();
//!
//! s1.insert(0, 1u8);
//! s2.insert(0, 2u8);
//! s3.insert(0, 3u8);
//!
//! let sk = Shamir { t: 2, n: 3 }.combine_shares::<Scalar>(&[Share(s1), Share(s2), Share(s3)]).unwrap();
//! let computed_pk = ProjectivePoint::GENERATOR * sk;
//! assert_eq!(computed_pk, pk1);
//! ```
//!
//! The output of the DKG is the same as if shamir secret sharing
//! had been run on the secret and sent to separate parties.
#![deny(
    missing_docs,
    unused_import_braces,
    unused_qualifications,
    unused_parens,
    unused_lifetimes,
    unconditional_recursion,
    unused_extern_crates,
    trivial_casts,
    trivial_numeric_casts
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use elliptic_curve;
pub use rand_core;
pub use vsss_rs;

mod error;
mod round1;
mod round2;
mod round3;
mod round4;
mod round5;

use elliptic_curve::{group::GroupEncoding, Field, Group, PrimeField};
use rand_core::SeedableRng;
use serde::{
    de::{Error as DError, SeqAccess, Unexpected, Visitor},
    ser::{SerializeSeq, SerializeTuple},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Formatter,
    marker::PhantomData,
    num::NonZeroUsize,
};
use uint_zigzag::Uint;
use vsss_rs::{FeldmanVerifier, Pedersen, PedersenResult, PedersenVerifier, Share};

pub use error::*;

/// The parameters used by the DKG participants.
/// This must be the same for all of them otherwise the protocol
/// will abort.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Parameters<G: Group + GroupEncoding + Default> {
    threshold: usize,
    limit: usize,
    #[serde(serialize_with = "serialize_g", deserialize_with = "deserialize_g")]
    message_generator: G,
    #[serde(serialize_with = "serialize_g", deserialize_with = "deserialize_g")]
    blinder_generator: G,
}

impl<G: Group + GroupEncoding + Default> Default for Parameters<G> {
    fn default() -> Self {
        Self {
            threshold: 0,
            limit: 0,
            message_generator: G::identity(),
            blinder_generator: G::identity(),
        }
    }
}

impl<G: Group + GroupEncoding + Default> Parameters<G> {
    /// Create regular parameters with the message_generator as the default generator
    /// and a random blinder_generator
    pub fn new(threshold: NonZeroUsize, limit: NonZeroUsize) -> Self {
        let message_generator = G::generator();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&message_generator.to_bytes().as_ref()[0..32]);
        let rng = rand_chacha::ChaChaRng::from_seed(seed);
        Self {
            threshold: threshold.get(),
            limit: limit.get(),
            message_generator: G::generator(),
            blinder_generator: G::random(rng),
        }
    }

    /// Use the provided parameters
    pub fn with_generators(
        threshold: NonZeroUsize,
        limit: NonZeroUsize,
        message_generator: G,
        blinder_generator: G,
    ) -> Self {
        Self {
            threshold: threshold.get(),
            limit: limit.get(),
            message_generator,
            blinder_generator,
        }
    }
}

/// A DKG participant. Maintains state information for each round
#[derive(Serialize, Deserialize)]
pub struct Participant<G: Group + GroupEncoding + Default> {
    id: usize,
    #[serde(bound(serialize = "PedersenResult<G::Scalar, G>: Serialize"))]
    #[serde(bound(deserialize = "PedersenResult<G::Scalar, G>: Deserialize<'de>"))]
    components: PedersenResult<G::Scalar, G>,
    threshold: usize,
    limit: usize,
    round: Round,
    #[serde(
        serialize_with = "serialize_scalar",
        deserialize_with = "deserialize_scalar"
    )]
    secret_share: G::Scalar,
    #[serde(serialize_with = "serialize_g", deserialize_with = "deserialize_g")]
    public_key: G,
    round1_broadcast_data: BTreeMap<usize, Round1BroadcastData<G>>,
    round1_p2p_data: BTreeMap<usize, Round1P2PData>,
    valid_participant_ids: BTreeSet<usize>,
}

/// Valid rounds
#[derive(Copy, Clone, Serialize, Deserialize)]
enum Round {
    One,
    Two,
    Three,
    Four,
    Five,
}

/// Broadcast data from round 1 that should be sent to all other participants
#[derive(Clone, Serialize, Deserialize)]
pub struct Round1BroadcastData<G: Group + GroupEncoding + Default> {
    #[serde(serialize_with = "serialize_g", deserialize_with = "deserialize_g")]
    message_generator: G,
    #[serde(serialize_with = "serialize_g", deserialize_with = "deserialize_g")]
    blinder_generator: G,
    #[serde(
        serialize_with = "serialize_g_vec",
        deserialize_with = "deserialize_g_vec"
    )]
    pedersen_commitments: Vec<G>,
}

/// Echo broadcast data from round 2 that should be sent to all valid participants
#[derive(Clone, Serialize, Deserialize)]
pub struct Round2EchoBroadcastData {
    valid_participant_ids: BTreeSet<usize>,
}

/// Broadcast data from round 3 that should be sent to all valid participants
#[derive(Clone, Serialize, Deserialize)]
pub struct Round3BroadcastData<G: Group + GroupEncoding + Default> {
    #[serde(serialize_with = "serialize_g", deserialize_with = "deserialize_g")]
    message_generator: G,
    #[serde(
        serialize_with = "serialize_g_vec",
        deserialize_with = "deserialize_g_vec"
    )]
    commitments: Vec<G>,
}

/// Echo broadcast data from round 4 that should be sent to all valid participants
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Round4EchoBroadcastData<G: Group + GroupEncoding + Default> {
    /// The computed public key
    #[serde(serialize_with = "serialize_g", deserialize_with = "deserialize_g")]
    pub public_key: G,
}

/// Peer data from round 1 that should only be sent to a specific participant
#[derive(Clone, Serialize, Deserialize)]
pub struct Round1P2PData {
    #[serde(
        serialize_with = "serialize_share",
        deserialize_with = "deserialize_share"
    )]
    secret_share: Share,
    #[serde(
        serialize_with = "serialize_share",
        deserialize_with = "deserialize_share"
    )]
    blind_share: Share,
}

impl<G: Group + GroupEncoding + Default> Participant<G> {
    /// Create a new participant to generate a new key share
    pub fn new(id: NonZeroUsize, parameters: Parameters<G>) -> DkgResult<Self> {
        let mut rng = rand_core::OsRng;
        let secret = G::Scalar::random(&mut rng);
        let blinder = G::Scalar::random(&mut rng);
        Self::initialize(id, parameters, secret, blinder)
    }

    /// Create a new participant to generate a refresh share.
    /// This method enables proactive secret sharing.
    /// The difference between new and refresh is new generates a random secret
    /// where refresh uses zero as the secret which just alters the polynomial
    /// when added to the share generated from new but doesn't change the secret itself.
    ///
    /// The algorithm runs the same regardless of the value used for secret.
    ///
    /// Another approach is to just run the DKG with the same secret since a different
    /// polynomial will be generated from the share, however, this approach exposes the shares
    /// if an attacker obtains any traffic. Using zero is safer in this regard and only requires
    /// an addition to the share upon completion.
    pub fn refresh(id: NonZeroUsize, parameters: Parameters<G>) -> DkgResult<Self> {
        let blinder = G::Scalar::random(rand_core::OsRng);
        Self::initialize(id, parameters, G::Scalar::zero(), blinder)
    }

    fn initialize(
        id: NonZeroUsize,
        parameters: Parameters<G>,
        secret: G::Scalar,
        blinder: G::Scalar,
    ) -> DkgResult<Self> {
        let pedersen = Pedersen {
            t: parameters.threshold,
            n: parameters.limit,
        };
        let mut rng = rand_core::OsRng;
        let components = pedersen.split_secret(
            secret,
            Some(blinder),
            Some(parameters.message_generator),
            Some(parameters.blinder_generator),
            &mut rng,
        )?;

        if (components.verifier.generator.is_identity()
            | components.verifier.feldman_verifier.generator.is_identity())
            .unwrap_u8()
            == 1u8
        {
            return Err(Error::InitializationError("Invalid generators".to_string()));
        }
        if components
            .verifier
            .commitments
            .iter()
            .any(|c| c.is_identity().unwrap_u8() == 1u8)
            || components
            .verifier
            .feldman_verifier
            .commitments
            .iter()
            .any(|c| c.is_identity().unwrap_u8() == 1u8)
        {
            return Err(Error::InitializationError(
                "Invalid commitments".to_string(),
            ));
        }
        if components.secret_shares.iter().any(|s| s.is_zero())
            || components.blind_shares.iter().any(|s| s.is_zero())
        {
            return Err(Error::InitializationError("Invalid shares".to_string()));
        }
        Ok(Self {
            id: id.get(),
            components,
            threshold: parameters.threshold,
            limit: parameters.limit,
            round: Round::One,
            round1_broadcast_data: BTreeMap::new(),
            round1_p2p_data: BTreeMap::new(),
            secret_share: G::Scalar::zero(),
            public_key: G::identity(),
            valid_participant_ids: BTreeSet::new(),
        })
    }

    /// The identifier associated with this participant
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Computed secret share.
    /// This value is useless until all rounds have been run
    pub fn get_secret_share(&self) -> G::Scalar {
        self.secret_share
    }

    /// Computed public key
    /// This value is useless until all rounds have been run
    pub fn get_public_key(&self) -> G {
        self.public_key
    }
}

fn serialize_share<S: Serializer>(share: &Share, s: S) -> Result<S::Ok, S::Error> {
    if s.is_human_readable() {
        s.serialize_str(&base64_url::encode(share.as_ref()))
    } else {
        share.serialize(s)
    }
}

fn deserialize_share<'de, D: Deserializer<'de>>(d: D) -> Result<Share, D::Error> {
    struct ShareVisitor;

    impl<'de> Visitor<'de> for ShareVisitor {
        type Value = Share;

        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(f, "a base64 encoded string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: DError,
        {
            let bytes = base64_url::decode(v)
                .map_err(|_| DError::invalid_value(Unexpected::Str(v), &self))?;
            Ok(Share(bytes))
        }
    }

    if d.is_human_readable() {
        d.deserialize_str(ShareVisitor)
    } else {
        Share::deserialize(d)
    }
}

fn serialize_scalar<F: PrimeField, S: Serializer>(scalar: &F, s: S) -> Result<S::Ok, S::Error> {
    let v = scalar.to_repr();
    let vv = v.as_ref();
    if s.is_human_readable() {
        s.serialize_str(&base64_url::encode(vv))
    } else {
        let len = vv.len();
        let mut t = s.serialize_tuple(len)?;
        for vi in vv {
            t.serialize_element(vi)?;
        }
        t.end()
    }
}

fn deserialize_scalar<'de, F: PrimeField, D: Deserializer<'de>>(d: D) -> Result<F, D::Error> {
    struct ScalarVisitor<F: PrimeField> {
        marker: PhantomData<F>,
    }

    impl<'de, F: PrimeField> Visitor<'de> for ScalarVisitor<F> {
        type Value = F;

        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(f, "a byte sequence")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: DError,
        {
            let bytes = base64_url::decode(v)
                .map_err(|_| DError::invalid_value(Unexpected::Str(v), &self))?;
            let mut repr = F::default().to_repr();
            repr.as_mut().copy_from_slice(bytes.as_slice());
            let sc = F::from_repr(repr);
            if sc.is_some().unwrap_u8() == 1u8 {
                Ok(sc.unwrap())
            } else {
                Err(DError::custom("unable to convert to scalar".to_string()))
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut repr = F::default().to_repr();
            let mut i = 0;
            let len = repr.as_ref().len();
            while let Some(b) = seq.next_element()? {
                repr.as_mut()[i] = b;
                i += 1;
                if i == len {
                    let sc = F::from_repr(repr);
                    if sc.is_some().unwrap_u8() == 1u8 {
                        return Ok(sc.unwrap());
                    }
                }
            }
            Err(DError::custom("unable to convert to scalar".to_string()))
        }
    }

    let vis = ScalarVisitor {
        marker: PhantomData::<F>,
    };
    if d.is_human_readable() {
        d.deserialize_str(vis)
    } else {
        let repr = F::default().to_repr();
        let len = repr.as_ref().len();
        d.deserialize_tuple(len, vis)
    }
}

fn serialize_g<G: Group + GroupEncoding + Default, S: Serializer>(
    g: &G,
    s: S,
) -> Result<S::Ok, S::Error> {
    let v = g.to_bytes();
    let vv = v.as_ref();
    if s.is_human_readable() {
        s.serialize_str(&base64_url::encode(vv))
    } else {
        let mut t = s.serialize_tuple(vv.len())?;
        for b in vv {
            t.serialize_element(b)?;
        }
        t.end()
    }
}

fn deserialize_g<'de, G: Group + GroupEncoding + Default, D: Deserializer<'de>>(
    d: D,
) -> Result<G, D::Error> {
    struct GVisitor<G: Group + GroupEncoding + Default> {
        marker: PhantomData<G>,
    }

    impl<'de, G: Group + GroupEncoding + Default> Visitor<'de> for GVisitor<G> {
        type Value = G;

        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(f, "a base64 encoded string or tuple of bytes")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: DError,
        {
            let mut repr = G::Repr::default();
            let bytes = base64_url::decode(v)
                .map_err(|_| DError::invalid_value(Unexpected::Str(v), &self))?;
            repr.as_mut().copy_from_slice(bytes.as_slice());
            let res = G::from_bytes(&repr);
            if res.is_some().unwrap_u8() == 1u8 {
                Ok(res.unwrap())
            } else {
                Err(DError::invalid_value(Unexpected::Str(v), &self))
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut repr = G::Repr::default();
            let input = repr.as_mut();
            for i in 0..input.len() {
                input[i] = seq
                    .next_element()?
                    .ok_or_else(|| DError::invalid_length(input.len(), &self))?;
            }
            let res = G::from_bytes(&repr);
            if res.is_some().unwrap_u8() == 1u8 {
                Ok(res.unwrap())
            } else {
                Err(DError::invalid_value(Unexpected::Seq, &self))
            }
        }
    }

    let visitor = GVisitor {
        marker: PhantomData,
    };
    if d.is_human_readable() {
        d.deserialize_str(visitor)
    } else {
        let repr = G::Repr::default();
        d.deserialize_tuple(repr.as_ref().len(), visitor)
    }
}

fn serialize_g_vec<G: Group + GroupEncoding + Default, S: Serializer>(
    g: &Vec<G>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let v = g.iter().map(|p| p.to_bytes()).collect::<Vec<G::Repr>>();
    if s.is_human_readable() {
        let vv = v
            .iter()
            .map(|b| base64_url::encode(b.as_ref()))
            .collect::<Vec<String>>();
        vv.serialize(s)
    } else {
        let size = G::Repr::default().as_ref().len();
        let uint = uint_zigzag::Uint::from(g.len());
        let length_bytes = uint.to_vec();
        let mut seq = s.serialize_seq(Some(length_bytes.len() + size * g.len()))?;
        for b in &length_bytes {
            seq.serialize_element(b)?;
        }
        for c in &v {
            for b in c.as_ref() {
                seq.serialize_element(b)?;
            }
        }
        seq.end()
    }
}

fn deserialize_g_vec<'de, G: Group + GroupEncoding + Default, D: Deserializer<'de>>(
    d: D,
) -> Result<Vec<G>, D::Error> {
    struct NonReadableVisitor<G: Group + GroupEncoding + Default> {
        marker: PhantomData<G>,
    }

    impl<'de, G: Group + GroupEncoding + Default> Visitor<'de> for NonReadableVisitor<G> {
        type Value = Vec<G>;

        fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
            write!(f, "an array of bytes")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut buffer = [0u8; Uint::MAX_BYTES];
            let mut i = 0;
            while let Some(b) = seq.next_element()? {
                buffer[i] = b;
                if i == Uint::MAX_BYTES {
                    break;
                }
            }
            let bytes_cnt_size = Uint::peek(&buffer)
                .ok_or_else(|| DError::invalid_value(Unexpected::Bytes(&buffer), &self))?;
            let points = Uint::try_from(&buffer[..bytes_cnt_size])
                .map_err(|_| DError::invalid_value(Unexpected::Bytes(&buffer), &self))?;

            i = Uint::MAX_BYTES - bytes_cnt_size;
            let mut repr = G::Repr::default();
            {
                let r = repr.as_mut();
                r[..i].copy_from_slice(&buffer[bytes_cnt_size..]);
            }
            let repr_len = repr.as_ref().len();
            let mut out = Vec::with_capacity(points.0 as usize);
            while let Some(b) = seq.next_element()? {
                repr.as_mut()[i] = b;
                if i == repr_len {
                    i = 0;
                    let pt = G::from_bytes(&repr);
                    if pt.is_none().unwrap_u8() == 1u8 {
                        return Err(DError::invalid_value(Unexpected::Bytes(&buffer), &self));
                    }
                    out.push(pt.unwrap());
                    if out.len() == points.0 as usize {
                        break;
                    }
                }
                i += 1;
            }
            if out.len() != points.0 as usize {
                return Err(DError::invalid_length(out.len(), &self));
            }
            Ok(out)
        }
    }

    if d.is_human_readable() {
        let s = Vec::<String>::deserialize(d)?;
        let mut out = Vec::with_capacity(s.len());
        for si in &s {
            let mut repr = G::Repr::default();
            let bytes = base64_url::decode(si)
                .map_err(|_| DError::custom("unable to decode string to bytes".to_string()))?;
            repr.as_mut().copy_from_slice(bytes.as_slice());
            let pt = G::from_bytes(&repr);
            if pt.is_none().unwrap_u8() == 1u8 {
                return Err(DError::custom(
                    "unable to convert string to point".to_string(),
                ));
            }
            out.push(pt.unwrap());
        }
        Ok(out)
    } else {
        d.deserialize_seq(NonReadableVisitor {
            marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vsss_rs::{Shamir, Share};

    #[test]
    fn one_corrupted_party_k256() {
        one_corrupted_party::<k256::ProjectivePoint>()
    }

    #[test]
    fn one_corrupted_party_p256() {
        one_corrupted_party::<p256::ProjectivePoint>()
    }

    #[test]
    fn one_corrupted_party_curve25519() {
        one_corrupted_party::<vsss_rs::curve25519::WrappedRistretto>();
        one_corrupted_party::<vsss_rs::curve25519::WrappedEdwards>();
    }

    #[test]
    fn one_corrupted_party_bls12_381() {
        one_corrupted_party::<bls12_381_plus::G1Projective>();
        one_corrupted_party::<bls12_381_plus::G2Projective>();
    }

    fn one_corrupted_party<G: Group + GroupEncoding + Default>() {
        const THRESHOLD: usize = 2;
        const LIMIT: usize = 4;
        const BAD_ID: usize = 4;

        let threshold = NonZeroUsize::new(THRESHOLD).unwrap();
        let limit = NonZeroUsize::new(LIMIT).unwrap();
        let parameters = Parameters::<G>::new(threshold, limit);
        let mut participants = [
            Participant::<G>::new(NonZeroUsize::new(1).unwrap(), parameters).unwrap(),
            Participant::<G>::new(NonZeroUsize::new(2).unwrap(), parameters).unwrap(),
            Participant::<G>::new(NonZeroUsize::new(3).unwrap(), parameters).unwrap(),
            Participant::<G>::new(NonZeroUsize::new(4).unwrap(), parameters).unwrap(),
        ];

        let mut r1bdata = Vec::with_capacity(LIMIT);
        let mut r1p2pdata = Vec::with_capacity(LIMIT);
        for p in participants.iter_mut() {
            let (broadcast, p2p) = p.round1().expect("Round 1 should work");
            r1bdata.push(broadcast);
            r1p2pdata.push(p2p);
        }
        for p in participants.iter_mut() {
            assert!(p.round1().is_err());
        }

        // Corrupt bad actor
        for i in 0..THRESHOLD {
            r1bdata[BAD_ID - 1].pedersen_commitments[i] = G::identity();
        }

        let mut r2bdata = BTreeMap::new();

        for i in 0..LIMIT {
            let mut bdata = BTreeMap::new();
            let mut p2pdata = BTreeMap::new();

            let my_id = participants[i].get_id();
            for j in 0..LIMIT {
                let pp = &participants[j];
                let id = pp.get_id();
                if my_id == id {
                    continue;
                }
                bdata.insert(id, r1bdata[id - 1].clone());
                p2pdata.insert(id, r1p2pdata[id - 1][&my_id].clone());
            }
            let p = &mut participants[i];
            let res = p.round2(bdata, p2pdata);
            assert!(res.is_ok());
            if my_id == BAD_ID {
                continue;
            }
            r2bdata.insert(my_id, res.unwrap());
        }

        let mut r3bdata = BTreeMap::new();
        for p in participants.iter_mut() {
            if BAD_ID == p.get_id() {
                continue;
            }
            let res = p.round3(&r2bdata);
            assert!(res.is_ok());
            r3bdata.insert(p.get_id(), res.unwrap());
            assert!(p.round3(&r2bdata).is_err());
        }

        let mut r4bdata = BTreeMap::new();
        let mut r4shares = Vec::with_capacity(LIMIT);
        for p in participants.iter_mut() {
            if BAD_ID == p.get_id() {
                continue;
            }
            let res = p.round4(&r3bdata);
            assert!(res.is_ok());
            let bdata = res.unwrap();
            let share = p.get_secret_share();
            r4bdata.insert(p.get_id(), bdata);
            let mut pshare = share.to_repr().as_ref().to_vec();
            pshare.insert(0, p.get_id() as u8);
            r4shares.push(Share(pshare));
            assert!(p.round4(&r3bdata).is_err());
        }

        for p in &participants {
            if BAD_ID == p.get_id() {
                continue;
            }
            assert!(p.round5(&r4bdata).is_ok());
        }

        let res = Shamir {
            t: THRESHOLD,
            n: LIMIT,
        }
        .combine_shares::<G::Scalar>(&r4shares);
        assert!(res.is_ok());
        let secret = res.unwrap();

        assert_eq!(r4bdata[&1].public_key, G::generator() * secret);
    }
}
