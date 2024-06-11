use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    HookError(#[from] cw_controllers::HookError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid campaign time range")]
    InvalidCampaignTime {},

    #[error("This campaign has ended")]
    CampaignEnded {},
}
