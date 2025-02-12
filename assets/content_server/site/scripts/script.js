
async function loadHeader() {
  let templates = document.createElement('template');
  templates.innerHTML = await ( await fetch('/pages/header.html') ).text();

  var header = templates.content.querySelector('#header');
  document.body.prepend(header.content);
}

async function crashDrone(id) {
  let response = await fetch(`/crash_drone/${id}`, {
    method: "POST",
  });
  let body = await response.text();
  alert(body);
}

async function setPdr(p) {
  let pdr = parseFloat(p) / 100.0;
  let response = await fetch(`/set_pdr`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      pdr: pdr,
    }),
  });
  alert(`set pdr to: ${pdr}`);
}

