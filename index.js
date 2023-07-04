const express = require('express')
const { ethers } = require("ethers");
require('dotenv').config();

const app = express()
const port = 8080

const exampleContract = "0x3C2982CA260e870eee70c423818010DfeF212659";

const RPC_URLS = {
    1: process.env.MAINNET_RPC,
    10: process.env.OPTIMISM_RPC,
    137: process.env.POLYGON_RPC,
    250: process.env.FANTOM_RPC,
    42161: process.env.ARBITRUM_RPC,
}

const explorer_base = {
  1: "https://etherscan.io/tx/",
  10: "https://optimistic.etherscan.io/tx/",
  137: "https://polygonscan.io/tx/",
  250: "https://ftmscan.com/tx/",
  42161: "https://arbiscan.io/tx/"
}

const abi = [
  "event RoleGranted(bytes32 indexed, address indexed, address indexed)",
  "event RoleRevoked(bytes32 indexed, address indexed, address indexed)",
  "event RoleAdminChanged(bytes32 indexed, bytes32 indexed, bytes32 indexed)"
]

app.get('/:chainId/:address', async (req, res) => {
  let pageView = `<!doctype HTML><html style=\"background:#C8AD7F\">
  <style>
  h1, h2 {
    text-align: center;
    font-family: monospace;
  }

  h3 {
    margin-block-end: 0em;
  }

  h4 {
    margin-block-end: 0em;
    margin-block-start: 0.5em

  }

  p, a, h5 {
    font-family: monospace;
  }
  </style>
  <h1>RoleRetriever</h1>
  <h2>Role-based Access Control Viewer for Smart Contracts</h2>
  <h2>${req.params.address}</h2>`

  try {
    let roles = {}

    console.log(`Provider: ${RPC_URLS[Number(req.params.chainId)]}`)

    const provider = new ethers.JsonRpcProvider(RPC_URLS[Number(req.params.chainId)]);

    let contract = new ethers.Contract(req.params.address, abi, provider);
    
    let GrantlogFilter = contract.filters.RoleGranted();
    let logs = await contract.queryFilter(GrantlogFilter);

    logs.forEach(log => {
      // console.log(log);
      let role = log.topics[1]
      //account granted is 2nd topic
      let user = "0x" + log.topics[2].substring(26, log.topics[2].length)
      // console.log(log.topics[2]);
      if (roles[role] == undefined) {
        roles[role] = []
      }
    roles[role].push({
        user: user,
        tx: log.transactionHash
    })

    //  console.log(user);
    })

    // Get Role Admins
    let roleAdminFilter = contract.filters.RoleAdminChanged();
    let adminLogs = await contract.queryFilter(roleAdminFilter);
    console.log(adminLogs);

    let roleAdmins = {}
    adminLogs.forEach(adminLog => {
      roleAdmins[adminLog.topics[1]] = adminLog.topics[3]
    })

    Object.keys(roles).forEach(key => {
      pageView += `<h3>${key}</h3>`
      pageView += `<h4>Admin: ${roleAdmins[key] != undefined ? roleAdmins[key] : '0x0000000000000000000000000000000000000000000000000000000000000000'}</h4>`
      roles[key].forEach(user => {
        pageView += `<p style=\"margin-block-end:0em\">User: ${user.user}</p>`
        pageView += `<a href=\"${explorer_base[req.params.chainId] + user.tx}\" target=_blank>Tx: ${user.tx}</a></br>`
        pageView += "</br>"
      
      })
    })

    pageView += `</html>`

    res.send(pageView);
  } catch (Err) {
    let pageView = `
    <h2>Error Occured: Could not retrieve info for ${req.params.address}</h2>
    <h2>Please try again later or double-check your contract address</h2>
    </html>
    `
    
    res.send(pageView)
  }

})

app.listen(port, () => {
  console.log(`Program is istening on port ${port}`)
})