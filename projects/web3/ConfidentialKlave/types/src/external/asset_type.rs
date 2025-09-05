// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::share::CkNetwork;
use crate::share::MultiChainAccountId;
use crate::share::NetworkType;
use anyhow::{bail, Result};
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::hash::Hash;

pub type ContractAddress = [u8; 20];

// as memeber of other structs: TxTransferInfo, Balance,etc
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Sequence)]
pub enum AssetType {
    ETH,
    USDT,
    USDC,
    BTC,
    BSC,
}

impl AssetType {
    pub fn config(&self) -> Box<dyn AssetConfig> {
        match self {
            AssetType::ETH => Box::new(Eth),
            AssetType::USDT => Box::new(Usdt),
            AssetType::USDC => Box::new(Usdc),
            AssetType::BTC => Box::new(Btc),
            AssetType::BSC => Box::new(Bsc),
        }
    }

    pub fn is_ethereum_chain(&self) -> bool {
        matches!(self, AssetType::ETH | AssetType::USDT | AssetType::USDC)
    }

    pub fn is_erc20(&self) -> bool {
        matches!(self, AssetType::USDT | AssetType::USDC)
    }

    pub fn is_bitcoin_chain(&self) -> bool {
        matches!(self, AssetType::BTC)
    }

    pub fn is_bsc_chain(&self) -> bool {
        matches!(self, AssetType::BSC)
    }

    pub fn is_evm_compatible(&self) -> bool {
        matches!(
            self,
            AssetType::ETH | AssetType::USDT | AssetType::USDC | AssetType::BSC
        )
    }

    pub fn all_eth_assets() -> Vec<AssetType> {
        vec![AssetType::ETH, AssetType::USDT, AssetType::USDC]
    }

    pub fn eth_native_type() -> AssetType {
        AssetType::ETH
    }

    pub fn erc20_types() -> Vec<AssetType> {
        vec![AssetType::USDT, AssetType::USDC]
    }

    pub fn all_btc_assets() -> Vec<AssetType> {
        vec![AssetType::BTC]
    }

    pub fn all_bsc_assets() -> Vec<AssetType> {
        vec![AssetType::BSC]
    }

    pub fn all_assets() -> Vec<AssetType> {
        vec![
            AssetType::ETH,
            AssetType::USDT,
            AssetType::USDC,
            AssetType::BTC,
            AssetType::BSC,
        ]
    }

    pub fn all_evm_compatible_assets() -> Vec<AssetType> {
        vec![
            AssetType::ETH,
            AssetType::USDT,
            AssetType::USDC,
            AssetType::BSC,
        ]
    }

    pub fn as_ck_network(&self, network_type: NetworkType) -> CkNetwork {
        match self {
            AssetType::ETH | AssetType::USDT | AssetType::USDC => CkNetwork::Eth(network_type),
            AssetType::BTC => CkNetwork::Btc(network_type),
            AssetType::BSC => CkNetwork::Bsc(network_type),
        }
    }

    pub fn associated_with_multichain_account_id(
        &self,
        multichain_account_id: &MultiChainAccountId,
    ) -> bool {
        match multichain_account_id {
            MultiChainAccountId::Eth(_) => self.is_ethereum_chain(),
            MultiChainAccountId::Btc(_) => self.is_bitcoin_chain(),
        }
    }

    pub fn filter_assets_by_ck_network(ck_network: &CkNetwork) -> Vec<AssetType> {
        AssetType::all_assets()
            .into_iter()
            .filter(|&asset| asset.as_ck_network(ck_network.network_type()) == *ck_network)
            .collect()
    }
}

impl TryFrom<String> for AssetType {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "ETH" => Ok(AssetType::ETH),
            "USDT" => Ok(AssetType::USDT),
            "USDC" => Ok(AssetType::USDC),
            "BTC" => Ok(AssetType::BTC),
            "BSC" => Ok(AssetType::BSC),
            _ => bail!("invalid asset type"),
        }
    }
}
impl From<AssetType> for String {
    fn from(value: AssetType) -> Self {
        match value {
            AssetType::ETH => "ETH".to_string(),
            AssetType::USDT => "USDT".to_string(),
            AssetType::USDC => "USDC".to_string(),
            AssetType::BTC => "BTC".to_string(),
            AssetType::BSC => "BSC".to_string(),
        }
    }
}

pub trait AssetConfig: Send + Sync {
    fn currency_id(&self) -> String;
    fn decimals(&self) -> u32;
    fn contract_address(&self, network_type: &NetworkType) -> Option<ContractAddress>;
    fn fee_asset_type(&self) -> AssetType;
    fn is_erc20(&self) -> bool {
        self.contract_address(&NetworkType::Mainnet).is_some()
    }
}

#[derive(Copy, Clone)]
pub struct Eth;
impl AssetConfig for Eth {
    fn currency_id(&self) -> String {
        "ethereum".to_string()
    }
    fn decimals(&self) -> u32 {
        18
    }
    fn contract_address(&self, _network_type: &NetworkType) -> Option<ContractAddress> {
        None
    }
    fn fee_asset_type(&self) -> AssetType {
        AssetType::ETH
    }
}

#[derive(Copy, Clone)]
pub struct Usdt;
impl AssetConfig for Usdt {
    fn currency_id(&self) -> String {
        "tether".to_string()
    }
    fn decimals(&self) -> u32 {
        6
    }
    fn contract_address(&self, network_type: &NetworkType) -> Option<ContractAddress> {
        match network_type {
            NetworkType::Mainnet => Some(
                hex::decode("dac17f958d2ee523a2206206994597c13d831ec7")
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or_default(),
            ),
            NetworkType::Testnet => Some(
                // sepolia
                hex::decode("aA8E23Fb1079EA71e0a56F48a2aA51851D8433D0")
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or_default(),
            ),
        }
    }
    fn fee_asset_type(&self) -> AssetType {
        AssetType::ETH
    }
}

#[derive(Copy, Clone)]
pub struct Usdc;
impl AssetConfig for Usdc {
    fn currency_id(&self) -> String {
        "usd-coin".to_string()
    }
    fn decimals(&self) -> u32 {
        6
    }
    fn is_erc20(&self) -> bool {
        true
    }
    fn contract_address(&self, network_type: &NetworkType) -> Option<ContractAddress> {
        match network_type {
            NetworkType::Mainnet => Some(
                hex::decode("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or_default(),
            ),
            NetworkType::Testnet => Some(
                // sepolia
                hex::decode("2B0974b96511a728CA6342597471366D3444Aa2a")
                    .unwrap_or_default()
                    .try_into()
                    .unwrap_or_default(),
            ),
        }
    }
    fn fee_asset_type(&self) -> AssetType {
        AssetType::ETH
    }
}

#[derive(Copy, Clone)]
pub struct Btc;
impl AssetConfig for Btc {
    fn currency_id(&self) -> String {
        "bitcoin".to_string()
    }
    fn decimals(&self) -> u32 {
        8
    }
    fn contract_address(&self, _network_type: &NetworkType) -> Option<ContractAddress> {
        None
    }
    fn fee_asset_type(&self) -> AssetType {
        AssetType::BTC
    }
}

#[derive(Copy, Clone)]
pub struct Bsc;
impl AssetConfig for Bsc {
    fn currency_id(&self) -> String {
        "binancecoin".to_string()
    }
    fn decimals(&self) -> u32 {
        18
    }
    fn contract_address(&self, _network_type: &NetworkType) -> Option<ContractAddress> {
        None
    }
    fn fee_asset_type(&self) -> AssetType {
        AssetType::BSC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enum_iterator::all;

    #[test]
    fn it_works() {
        // ierate over all asset types
        for token in all::<super::AssetType>() {
            let config = token.config();
            println!("currency_id: {:?}", config.currency_id());
            println!("decimals: {:?}", config.decimals());
            println!("is_erc20: {:?}", config.is_erc20());
            println!(
                "contract_address: {:?}",
                config.contract_address(&NetworkType::Mainnet)
            );
        }
    }
}
