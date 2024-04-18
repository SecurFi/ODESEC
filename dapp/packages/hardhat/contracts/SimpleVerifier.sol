//SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import {ControlID, RiscZeroGroth16Verifier} from "risc0/groth16/RiscZeroGroth16Verifier.sol";

contract SimpleVerifier is RiscZeroGroth16Verifier {
    constructor() RiscZeroGroth16Verifier(ControlID.CONTROL_ID_0, ControlID.CONTROL_ID_1) {}
}
