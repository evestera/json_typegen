import { polyfill } from './polyfill';

const $ = (id) => document.getElementById(id);

polyfill().then(() => import("../../json_typegen_wasm/pkg")).then(module => {
  const render = () => {
    const typename = $('typename').value;
    let input = $('input').value;
    if (input === '') {
      input = '{}';
    }
    const options = ({
      "output_mode": $('outputmode').value,
      "property_name_format": $('propertynameformat').value,
      "unwrap": $('unwrap').value,
    });
    const result = module.run(typename, input, JSON.stringify(options));
    $('target').innerHTML = result
        .replace(/&/g,'&amp;')
        .replace(/</g,'&lt;')
        .replace(/>/g,'&gt;');
  };

  $('typename').onkeyup = render;
  $('input').onkeyup = render;
  $('outputmode').onchange = render;
  $('propertynameformat').onchange = render;
  $('unwrap').onkeyup = render;

  render();
});
