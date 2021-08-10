// its 3am lmao

const endpoint = "/cdp_api";
const fetch_ival = 5000;
let per_broker = {};
let msgs = [];

// fetch the goods
function fetch_the_goods() {
  fetch(endpoint + "/messages/sensor")
    .then(resp => resp.json()).then(function(json) {
      msgs = json;
      per_broker = {}
      for (let msg of msgs) {
        console.log(msg);
        let bid = msg["broker_id"];
        console.log(bid);
        if (!per_broker.hasOwnProperty(bid)) {
          per_broker[bid] = [];
        }
        per_broker[bid].push(msg);
      }
    });
  console.log("fetched", msgs.length);
}

setInterval(fetch_the_goods, fetch_ival);
