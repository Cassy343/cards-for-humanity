// Downloads card packs from https://github.com/crhallberg/json-against-humanity
// run via node ./download_cards.js
// run with 'official' option to exclude custom cards


const fs = require('fs');
const http = require('https');


async function main() {

    let json_compat = JSON.parse(await get('https://raw.githubusercontent.com/crhallberg/json-against-humanity/latest/cah-all-compact.json'));

    let packs = [];
    for (let pack of json_compat.packs) {
        pack.white = pack.white.map((index) =>
            Object.assign({}, {
                text: json_compat.white[index]
            }, {
                pack: packs.length
            })
        );
        pack.black = pack.black.map((index) =>
            Object.assign({},
                json_compat.black[index], {
                    pack: packs.length
                }
            )
        );
        packs.push(pack);
    }

    if (process.argv[2] == 'official')
        packs = packs.filter((p) => p.official);
    
    fs.mkdirSync('./run/cards/official', {
        recursive: true
    });
    fs.mkdirSync('./run/cards/custom', {
        recursive: true
    });


    for (pack of packs) {
        fs.writeFileSync(`./run/cards/${pack.official ? 'official' : 'custom' }/${pack.name.replace('/', ' ')}.json`, JSON.stringify(pack));
    }
}

main().then(() => {})


function get(url) {
    return new Promise((res, rej) => {
        http.get(url, (req) => {
            let chunk = '';
            req.on('data', (c) => chunk += c);
            req.on('error', (e) => rej(e));
            req.on('end', () => res(chunk))
        });
    })
}