use bytes::BufMut;
use pumpkin_data::packet::clientbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::packet;

use crate::{ClientPacket, KnownPack, bytebuf::ByteBufMut};

#[packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct CKnownPacks<'a> {
    pub known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    pub fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
}

impl ClientPacket for CKnownPacks<'_> {
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_list::<KnownPack>(self.known_packs, |p, v| {
            p.put_string(v.namespace);
            p.put_string(v.id);
            p.put_string(v.version);
        });
    }
}
