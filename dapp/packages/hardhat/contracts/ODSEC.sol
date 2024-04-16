//SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

// Useful for debugging. Remove when deploying to a live network.
import "hardhat/console.sol";

// Use openzeppelin to inherit battle-tested implementations (ERC20, ERC721, etc)
// import "@openzeppelin/contracts/access/Ownable.sol";

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";

/**
 * A smart contract that allows changing a state variable of the contract and tracking the changes
 * It also allows the owner to withdraw the Ether in the contract
 * @author BuidlGuidl
 */
contract ODSEC {
    IRiscZeroVerifier public verifier;
    /// @notice Image ID of the only zkVM binary to accept verification from.
    bytes32 public imageId;
    // State Variables
    address public immutable owner;
    uint256 public totalProjects;
    bytes public constant MAGIC = "ODSEC";

    struct ProjectData {
        address owner;
        address[] contracts;
        string domain;
        string contact;
    }

    mapping(uint256 => ProjectData) public projects;
    mapping(bytes32 => uint256) public projectIds;

    // Constructor: Called once on contract deployment
    // Check packages/hardhat/deploy/00_deploy_your_contract.ts
    constructor(address _owner, IRiscZeroVerifier _verifier, bytes32 _imageId) {
        owner = _owner;
        verifier = _verifier;
        imageId = _imageId;
    }

    modifier isOwner() {
        // msg.sender: predefined variable that represents address of the account that called the current function
        require(msg.sender == owner, "Not the Owner");
        _;
    }

    function updateImageId(bytes32 _imageId) public isOwner {
        imageId = _imageId;
    }

    function addProject(string memory _domain, string memory _contact, address _owner, bytes memory receipt) public {
        (bytes memory journal, bytes32 postStateDigest, bytes memory seal) =
            abi.decode(receipt, (bytes, bytes32, bytes));

        require(_owner != address(0), "Invalid owner");
        require(bytes(_domain).length > 3, "Invalid domain");
        require(projectIdOfDomain(_domain) == 0, "Project already exists");
        require(verifier.verify(seal, imageId, postStateDigest, sha256(journal)));

        bytes32 challenge = makeChallenge(_domain, _owner);
        require(challenge == bytes32(journal), "Invalid challenge");

        totalProjects += 1;
        uint256 projectId = totalProjects;
        projects[projectId] = ProjectData(_owner, new address[](0), _domain, _contact);
        projectIds[keccak256(bytes(_domain))] = projectId;
    }

    /**
     * get the projectId by domain, if the domain is not registered, return 0
     */
    function projectIdOfDomain(string memory _domain) public view returns (uint256) {
        return projectIds[keccak256(bytes(_domain))];
    }

    function updateProject(uint256 projectId, string memory contact, address[] memory contracts) public {
        ProjectData storage project = projects[projectId];
        require(msg.sender == project.owner, "Only owner can update project");
        project.contact = contact;
        project.contracts = contracts;
    }

    function getProjectList(uint256 limit, uint256 offset) public view returns (ProjectData[] memory) {
        uint256 count = limit > totalProjects - offset ? totalProjects - offset : limit;
        ProjectData[] memory _projects = new ProjectData[](count);
        for (uint256 i = 0; i < count; i++) {
            _projects[i] = projects[offset + i];
        }
        return _projects;
    }

    function makeChallenge(string memory domain, address _owner) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(MAGIC, domain, _owner));
    }
}
