// its 3am lmao

const sel_broker = document.getElementById("sel_broker");
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
        let bid = msg["broker_id"];
        if (!per_broker.hasOwnProperty(bid)) {
          per_broker[bid] = [];
        }
        per_broker[bid].push(msg);
      }
    });
  console.log("fetched", msgs.length);
  update_brokers();
}

// updates the broker filter list
function update_brokers() {
  let prev = sel_broker.value;
  sel_broker.innerHTML = "";
  let opt = document.createElement("option");
  opt.value = "";
  opt.innerText = "[choose a broker]";
  sel_broker.appendChild(opt);
  for (const broker in per_broker) {
    let bopt = document.createElement("option");
    bopt.value = broker;
    bopt.innerText = broker.substr(0, 4);
    sel_broker.appendChild(bopt);
  }
  sel_broker.value = prev;
}

setInterval(fetch_the_goods, fetch_ival);
