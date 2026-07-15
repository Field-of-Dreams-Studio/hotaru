use core::iter::Peekable;
use crate::helper::*; 
use proc_macro::{Literal, TokenStream, TokenTree};

pub enum MWSlot{
    Concrete(Literal),
    Inherit 
}

impl MWSlot {
    pub fn get_next(
        stream: &mut Peekable<impl Iterator<Item = TokenTree>>
    ) -> Result<MWSlot, TokenStream> { 
        let next_token = stream.next();
        match next_token {
            Some(TokenTree::Ident(ident) => {
                return MWSlot::Concrete(Literal::string(&ident.to_string())); 
    }

    pub fn is_concrete(&self) -> bool {
        match self {
            MWSlot::Concrete(_) => true,
            MWSlot::Inherited => false,
        }
    }

    pub fn is_inherited(&self) -> bool {
        match self {
            MWSlot::Concrete(_) => false,
            MWSlot::Inherited => true,
        }
    }
} 

pub struct MWChain {
    slots: Vec<MWSlot>
} 

pub fn parse_mw_chain(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>
) -> Result<MWChain, TokenStream> {
    
} 

