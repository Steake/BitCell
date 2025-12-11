//! Standard library for BCL contracts

/// Standard library functions available in BCL contracts
pub mod functions {
    /// msg.sender - Returns the address of the caller (stored at 0x14)
    pub const MSG_SENDER_ADDR: u32 = 0x14;
    
    /// msg.value - Returns the amount sent with the transaction
    pub const MSG_VALUE_ADDR: u32 = 0x18;
    
    /// block.number - Returns the current block number
    pub const BLOCK_NUMBER_ADDR: u32 = 0x20;
    
    /// block.timestamp - Returns the current block timestamp
    pub const BLOCK_TIMESTAMP_ADDR: u32 = 0x28;
}

/// Memory layout for contract execution
pub mod memory {
    /// Function selector
    pub const FUNCTION_SELECTOR: u32 = 0x10;
    
    /// Function parameters start address (after built-in variables)
    pub const PARAMS_START: u32 = 0x30;
    
    /// Storage start address
    pub const STORAGE_START: u32 = 0x200;
    
    /// Temporary/stack memory start
    pub const STACK_START: u32 = 0x1000;
}

/// Common contract patterns
pub mod patterns {
    /// Simple ERC20-like token interface
    pub const TOKEN_CONTRACT: &str = r#"
contract Token {
    storage {
        balances: mapping(address => uint);
        total_supply: uint;
        owner: address;
    }
    
    function transfer(to: address, amount: uint) -> bool {
        let sender = msg.sender;
        require(balances[sender] >= amount, "Insufficient balance");
        
        balances[sender] = balances[sender] - amount;
        balances[to] = balances[to] + amount;
        
        return true;
    }
    
    function balance_of(account: address) -> uint {
        return balances[account];
    }
}
"#;

    /// Simple counter contract
    pub const COUNTER_CONTRACT: &str = r#"
contract Counter {
    storage {
        count: uint;
    }
    
    function increment() -> uint {
        count = count + 1;
        return count;
    }
    
    function decrement() -> uint {
        require(count > 0, "Counter underflow");
        count = count - 1;
        return count;
    }
    
    function get() -> uint {
        return count;
    }
}
"#;

    /// Simple escrow contract
    pub const ESCROW_CONTRACT: &str = r#"
contract Escrow {
    storage {
        depositor: address;
        beneficiary: address;
        amount: uint;
        released: bool;
    }
    
    function deposit(to: address, value: uint) -> bool {
        require(amount == 0, "Already deposited");
        
        depositor = msg.sender;
        beneficiary = to;
        amount = value;
        released = false;
        
        return true;
    }
    
    function release() -> bool {
        require(msg.sender == depositor, "Only depositor can release");
        require(!released, "Already released");
        
        released = true;
        return true;
    }
}
"#;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;

    #[test]
    fn test_compile_token_contract() {
        let result = compile(patterns::TOKEN_CONTRACT);
        if let Err(e) = &result {
            eprintln!("Compilation error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_counter_contract() {
        let result = compile(patterns::COUNTER_CONTRACT);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_escrow_contract() {
        let result = compile(patterns::ESCROW_CONTRACT);
        if let Err(e) = &result {
            eprintln!("Compilation error: {}", e);
        }
        assert!(result.is_ok());
    }
}
