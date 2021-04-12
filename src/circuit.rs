use std::collections::BTreeMap;

use slotmap::{new_key_type, SlotMap};

type PinNo = u32;

#[derive(Debug)]
pub enum Signal {
    Port { pin: PinNo },
    Wire,
}

new_key_type! {
    pub struct SignalKey;
}

#[derive(Debug)]
pub enum Gate {
    SumTerm {
        inp: Vec<SignalKey>,
        out: SignalKey,
    },
    ProdTerm {
        inp: Vec<SignalKey>,
        out: SignalKey,
    },
    Flop {
        d: SignalKey,
        q: SignalKey,
        qn: SignalKey,
    },
    Inverter {
        inp: SignalKey,
        out: SignalKey,
    }
}

#[derive(Debug)]
pub struct Circuit {
    signals: SlotMap<SignalKey, Signal>,
    ports: BTreeMap<PinNo, SignalKey>,
    gates: Vec<Gate>,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            signals: SlotMap::with_key(),
            gates: Vec::new(),
            ports: BTreeMap::new(),
        }
    }

    pub fn add_port(&mut self, pin: PinNo) -> SignalKey {
        let key = self.signals.insert(Signal::Port { pin });
        self.ports.insert(pin, key);
        key
    }

    pub fn new_wire(&mut self) -> SignalKey {
        self.signals.insert(Signal::Wire)
    }

    pub fn invert(&mut self, signal: SignalKey) -> SignalKey {
        let out_wire = self.new_wire();
        self.gates.push(Gate::Inverter { inp: signal, out: out_wire });
        out_wire
    }

    pub fn port(&self, pin: PinNo) -> Option<SignalKey> {
        self.ports.get(&pin).cloned()
    }
}
