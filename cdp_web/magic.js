// its 3am lmao

const sel_broker = document.getElementById("sel_broker");
const sel_topic = document.getElementById("sel_topic");
const chart_canvas = document.getElementById("chart_canvas");
const in_lastn = document.getElementById("in_lastn");
const endpoint = "/cdp_api";
const fetch_ival = 5000;
let per_broker = {};
let msgs = [];

// some colors lol
const colors = [
  "red", "green", "orange", "blue", "purple", "cyan", "magenta", "lime",
  "darkgreen"
];

let icor = 0;
function color() {
  let c = colors[icor % colors.length];
  icor++;
  return c;
}

// le chart
chart = new Chart(chart_canvas.getContext("2d"), {
  type: "line",
  responsive: true,
  data: {}
});

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
  u(true);
}

// auto fetch yay
setInterval(fetch_the_goods, fetch_ival);
fetch_the_goods();

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

// we a lil' flattening
function simpleflat(msg) {
  let flat = {};
  const supp = ["Temperature", "Humidity"]
  let sd = msg["payload"]["SensorData"];
  for (let kn in sd) {
    let inner = sd[kn];
    flat["topic"] = kn.toLowerCase();
    flat["sensor_id"] = inner["sensor_id"];
    for (let kkn in inner) {
      if (kkn == "sensor_id") continue;
      flat["value"] = inner[kkn];
    }
    flat["when"] = new Date(msg["constructed_when"]);
  }
  msg["flat"] = flat;
}

// a chart do-over!
function rechart(msgarr, lastn, topic, chart) {
  let labels = [];
  let datasets = [];
  let flats_per_sensor = {};
  let added = 0;
  let index = msgarr.length-1;
  while (index >= 0 && added < lastn) {
    let flat = msgarr[index]["flat"];
    if (flat["topic"] == topic) {
      let sid = flat["sensor_id"];
      if (!flats_per_sensor.hasOwnProperty(sid)) flats_per_sensor[sid] = [];
      flats_per_sensor[sid].push(flat);
      labels.push(flat["when"]);
      added++;
    }
    index--;
  }
  labels.sort();
  icor = 0;
  chart.data.datasets = [];
  for (let sensor_id in flats_per_sensor) {
    let flats = flats_per_sensor[sensor_id];
    flats.reverse();
    let dataset = {
      label: "Sensor #" + sensor_id,
      data: [],
      borderColor: color(),
      fill: false
    };
    let bytime = {};
    for (let sf of flats) {
      bytime[sf["when"]] = sf["value"];
    }
    for (let time of labels) {
      dataset.data.push(bytime[time]);
    }
    chart.data.datasets.push(dataset);
    console.log(dataset);
  }
  chart.update(0);
}

// the real goods: update stuff
function u(was_auto) {
  let broker = sel_broker.value;
  let lastn = in_lastn.value;
  let topic = sel_topic.value;
  let msgarr = per_broker[broker] || [];
  rechart(msgarr, lastn, topic, chart);
}
