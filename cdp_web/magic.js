// its 3am lmao

const sel_broker = document.getElementById("sel_broker");
const chart_canvas = document.getElementById("chart_canvas");
const in_lastn = document.getElementById("in_lastn");
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
        simpleflat(msg);
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

// auto fetch yay
setInterval(fetch_the_goods, fetch_ival);

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

let chart = new Chart(chart_canvas.getContext("2d"), {
  type: "line",
  responsive: true,
  data: {}
});

// we a lil' flattening
function simpleflat(msg) {
  let flat = {};
  const supp = ["Temperature", "Humidity"]
  let sd = msg["payload"]["SensorData"];
  for (let kn in sd) {
    flat["topic"] = kn.toLowerCase();
    flat["sensor_id"] = sd[kn]["sensor_id"];
    flat["value"] = sd[kn][flat["topic"]];
  }
  msg["flat"] = flat;
}

// a chart do-over!
function rechart(msgarr, lastn, topic) {
  let labels = [];

}
