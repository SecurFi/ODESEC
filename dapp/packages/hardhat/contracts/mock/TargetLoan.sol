//SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface ILoanReceiver {
    function receiveFlashloan(IERC20 token, uint256 amount) external;
}

contract TargetLoan {
    address public owner;

    constructor(address _owner) {
        owner = _owner;
    }

    modifier isOwner() {
        // msg.sender: predefined variable that represents address of the account that called the current function
        require(msg.sender == owner, "Not the Owner");
        _;
    }

    function flashloan(IERC20 token, uint256 amount, address receiver) external {
        uint256 balBefore = token.balanceOf(address(this));
        require(amount <= balBefore);
        uint256 feeAmount = amount * 5 / 10000;
        ILoanReceiver(receiver).receiveFlashloan(token, amount);
        uint256 balAfter = token.balanceOf(address(this));
        require(balAfter >= balBefore + feeAmount);
    }

    function withdraw(IERC20 token, uint256 amount, address to) external isOwner {
        token.transfer(to, amount);
    }

    function deposit(IERC20 token, uint256 amount) external {
        token.transferFrom(msg.sender, address(this), amount);
    }

    // changeOwner: function to change the owner of the contract
    // there is a bug here, the new owner can be any address
    function changeOwner(address newOwner) external {
        owner = newOwner;
    }
}
