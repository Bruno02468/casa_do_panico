// its 3am lmao

const endpoint = "/cdp_api";
const fetch_ival = 2000;
let per_broker = {};
let msgs = [];

// fetch the goods
function fetch_the_goods() {
  fetch(endpoint + "/messages/sensor")
    .then(resp => resp.json()).then(function(json) {
      msgs = json;
      per_broker = {}
      for (msg in msgs) {
        const bid = msg["broker_id"];
        if (!per_broker.hasOwnProperty(bid)) {
          per_broker[bid] = [];
        }
        per_broker[bid].push(msg);
      }
    });
  console.log("fetched", msgs.length);
}

setInterval(fetch_the_goods, fetch_ival);
