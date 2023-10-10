const { ethers } = require("ethers");
const axios = require('axios');
require('dotenv').config();

const exampleContract = "0x3432b6a60d23ca0dfca7761b7ab56459d9c964d0";

const API_KEYS = {
  1: "7XTZXCQH8I5S9TT6AZCSFQT64I3YTK6E8S"
}

const eventSigs = {
    "roleGranted": "0x2f8788117e7eff1d82e926ec794901d17c78024a50270940304540a733656f0d",
    "roleRevoked": "0xf6391f5c32d9c69d2a47ea670b442974b53935d1edc7fd64eb21e047a839171b",
}

const explorer_base = {
  1: "https://api.etherscan.io/api",
  10: "https://api.optimistic.etherscan.io/",
  137: "https://api.polygonscan.io/tx/",
  250: "https://api.ftmscan.com/tx/",
  42161: "https://api.arbiscan.io/tx/"
}

async function main() {
    let events = []
    let roles = {}

    try {

        let roleGrantedEvents = await axios.get(explorer_base[1], {params: {
            module: "logs",
            action: "getLogs",
            address: exampleContract,
            topic0: eventSigs["roleGranted"],
            apikey: API_KEYS[1]
        }});

        for (let x = 0; x < roleGrantedEvents.data.result.length; x++) {
            events.push({
                "timestamp": roleGrantedEvents.data.result[x].timeStamp,
                "role": roleGrantedEvents.data.result[x].topics[1],
                "event": "roleGranted",
                "address": roleGrantedEvents.data.result[x].topics[2]
            })
        }

        let roleRevokedEvents = await axios.get(explorer_base[1], {params: {
            module: "logs",
            action: "getLogs",
            address: exampleContract,
            topic0: eventSigs["roleRevoked"],
            apikey: API_KEYS[1]
        }});


        // console.log(roleGrantedEvents.data.result);
        for (let x = 0; x < roleRevokedEvents.data.result.length; x++) {
            events.push({
                "timestamp": roleRevokedEvents.data.result[x].timeStamp,
                "role": roleRevokedEvents.data.result[x].topics[1],
                "event": "roleRevoked",
                "address": roleRevokedEvents.data.result[x].topics[2]
            })
        }

        //Sort By timestamp
        events.sort((a, b) => {
            return a.timestamp - b.timestamp;
         });

       
        //Create a list of keys of addresses marking each as valid
        for (let x = 0; x < events.length; x++) {
            if (roles[events[x].role] == undefined) roles[events[x].role] = {}

            if (events[x].event == "roleGranted" && events[x].address != undefined) {
                roles[events[x].role][events[x].address] = true;
            }

            //If their role was revoked then mark for deletion
            else if (events[x].event == "roleRevoked") {
                if (roles[events[x].role][events[x].address] == true && events[x].address != undefined) {
                    // console.log(roles[events[x].role])
                    // console.log(`revoking access for: ${events[x].address}`)
                    // console.log(`Role revoking from: ${events[x].role}`)
                    // console.log(`Address being Revoked: ${events[x].address}`)
                    roles[events[x].role][events[x].address] = false;
                }
            }
        }

        // console.log(roles)

        let finalRoles = {}
        for (const role in roles) {
            console.log(role)
            finalRoles[role] = []
            for (const address in role) {
                if (role[address]) {
                    finalRoles[role].push(address);
                }
            }
        }

        console.log(finalRoles)

        
      } catch (Err) {
        console.log(Err)
      }

}

main()

