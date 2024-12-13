# Changelog

## 0.4.0 (2024-12-13)


### Features

* add args for logging and broker connection (not tls yet) ([2c40e76](https://github.com/kaans/mqtli/commit/2c40e76a531f749e215fa96efef5920070bcfa3f))
* add basic mqtt event loop ([980b1e6](https://github.com/kaans/mqtli/commit/980b1e6c2417e56d2f1e400fdf247aff3ce22ee8))
* add choice for tls version 1.2 or 1.3 or both ([4709a02](https://github.com/kaans/mqtli/commit/4709a028f79eb9479d1ccacd648cbc1efd8eab08))
* add config file for subscribing to topics ([6658bdf](https://github.com/kaans/mqtli/commit/6658bdfee99df80611bde91b4a64cc2d35833859))
* add config for multiple outputs per topic and an output to the console ([b954fe8](https://github.com/kaans/mqtli/commit/b954fe8a6c93066d19b55fa71a023a14db16af35))
* add enums for protobuf ([12c1071](https://github.com/kaans/mqtli/commit/12c107182456e3b352b1f43e6f2bf91c4323d155))
* add formatted output of received messages ([f6af5dd](https://github.com/kaans/mqtli/commit/f6af5ddad39936a64b831b099fc5ca2e45921b8a))
* add lat config ([c9f0413](https://github.com/kaans/mqtli/commit/c9f0413b92d52c96a99944f7dd76a2c14a5492af))
* add main config ([e8825c0](https://github.com/kaans/mqtli/commit/e8825c06399f6cdcb8fa3c354cd91e61633b2a05))
* add more output formats to payload type protobuf ([1ddf7f4](https://github.com/kaans/mqtli/commit/1ddf7f46f08a9c474833f45a5e913eb47460554c))
* add more output formats to payload type text ([377a0a7](https://github.com/kaans/mqtli/commit/377a0a78eb2e6eaa63de6f12973c2c27b78abaef))
* add option to output raw bytes as lossy utf8 ([efa72e2](https://github.com/kaans/mqtli/commit/efa72e2475454f68ec348973151470ef721bc76f))
* add option to parse raw bytes for json as hex or base64 ([cf7e991](https://github.com/kaans/mqtli/commit/cf7e99110beef832816e7e13ddd455314be361ba))
* add option to parse raw bytes for yaml as hex or base64 ([5626a69](https://github.com/kaans/mqtli/commit/5626a69b4e7c48ebd9f42254671e5eee14d0d891))
* add output converter for text and raw ([b09b6f3](https://github.com/kaans/mqtli/commit/b09b6f3b3c5ce9915bd58e44474a85fb42b83bb1))
* add output formats ([a8465ed](https://github.com/kaans/mqtli/commit/a8465edcb7c077f772c1422d566e541a09d248a4))
* add output to file ([5e89582](https://github.com/kaans/mqtli/commit/5e89582fee19ec9e3093afbd666037c873417905))
* add payload type to config ([fb162cb](https://github.com/kaans/mqtli/commit/fb162cb6293fd391499d47ac878d83a96f41929c))
* add periodic trigger job ([8956cc1](https://github.com/kaans/mqtli/commit/8956cc1a2e4761c5c84d2f6538fa764c4ac91ab6))
* add prepend and append values for file output ([b7189d0](https://github.com/kaans/mqtli/commit/b7189d0642174bc815551fa067a6167feda6523d))
* add publish triggers ([a2ee0ae](https://github.com/kaans/mqtli/commit/a2ee0aeeade523daa662ce14bd238d0a5a4c6ba7))
* add raw byte conversion to text payload ([8662360](https://github.com/kaans/mqtli/commit/866236069b91082db742f35fa89971e872864f36))
* add raw input type ([33aa36f](https://github.com/kaans/mqtli/commit/33aa36f76c0538e34d71880c3543f0f7b5638ff9))
* add raw output format ([e9c7d5c](https://github.com/kaans/mqtli/commit/e9c7d5c619db5eccce21e023b02939f44328d891))
* add validation to config ([4cf7680](https://github.com/kaans/mqtli/commit/4cf7680b5ad67177bbdb5b4fafdb7f99de07b7a2))
* added json and yaml input formats ([04ae718](https://github.com/kaans/mqtli/commit/04ae718f8ea150c04a83804ccc00ee37f2852b8e))
* added json converter for protobuf ([2f4788e](https://github.com/kaans/mqtli/commit/2f4788e731a5fd18e9bfa57a2a370daece058cbf))
* added remaining formats to payload type and inputs ([7fdbe5c](https://github.com/kaans/mqtli/commit/7fdbe5cfa13217262985919239d390386005c916))
* change all payload formats to native types and convert between them ([ddbe24c](https://github.com/kaans/mqtli/commit/ddbe24cf629520f93970c969351f06fe3d1445df))
* **config:** make the presence of the config file optional and use defaults ([3397429](https://github.com/kaans/mqtli/commit/3397429f2cc14c09636155b0784d3d52d560ed80))
* **config:** remove short flags for help and version, rename host flag to -h and add mqtt version flag -v ([a36b71b](https://github.com/kaans/mqtli/commit/a36b71b1c565d296f79a587ae2fbb0ff62e25119))
* **console:** use color to output to console and for logging ([9ec0530](https://github.com/kaans/mqtli/commit/9ec0530d563eb66f7a2dc569c0a286587aa2a1b6))
* convert payload protobuf from raw, hex, and base64 ([6cefb24](https://github.com/kaans/mqtli/commit/6cefb240e1ae5890b089ac22fa65bfaaaa07924f))
* extract config to module and read from args and file ([2fcfb64](https://github.com/kaans/mqtli/commit/2fcfb646f9c59acde6edf6fa2c76218b372d75e2))
* extract mqtt service into separate file ([70aafd1](https://github.com/kaans/mqtli/commit/70aafd15ae15df9248924a2ec1d8b43b5f725d14))
* implement payload parsers for protobuf and plain text ([973ec87](https://github.com/kaans/mqtli/commit/973ec8723d077e63cf515f3078c8c7db7fd9e05a))
* improved error handling ([b69c354](https://github.com/kaans/mqtli/commit/b69c354ec9ae7cf5c7d31ebc663eca74b0c4ffeb))
* listen to exit signal and shutdown gracefully ([857e372](https://github.com/kaans/mqtli/commit/857e37252a580c0f791207e13d8cda409b84c9a2))
* make config file optional ([f8c42b4](https://github.com/kaans/mqtli/commit/f8c42b44e326bdb5df25d7d5b07b1a0c71faf16a))
* move conversion between formats to single files ([cc90000](https://github.com/kaans/mqtli/commit/cc900003e22f2d3c4ad55a3e9cec1f7cfcd3601c))
* moved output config to subscription ([0ed1351](https://github.com/kaans/mqtli/commit/0ed13518153c3d614ffbf3d1a2f8f25b835ef70f))
* mqtli-1 Add TLS support ([7c5ee8c](https://github.com/kaans/mqtli/commit/7c5ee8c5c5f072d81790c263c678650b66dbf7ac))
* mqtli-2 Add authentication via TLS client certificates ([252bd60](https://github.com/kaans/mqtli/commit/252bd6062ef7f35b024e0bfc95b22d9bdb6ae3da))
* **mqtt:** create mqtt v311 client ([e35a98f](https://github.com/kaans/mqtli/commit/e35a98f5d63c008d4cedf9eb82eb25b51aa6388e))
* **mqtt:** enable websocket feature for rumqttc ([42eb5b8](https://github.com/kaans/mqtli/commit/42eb5b839746b785f91a2eafe84b9e301392f980))
* **mqtt:** support websockets, including tls ([#28](https://github.com/kaans/mqtli/issues/28)) ([42eb5b8](https://github.com/kaans/mqtli/commit/42eb5b839746b785f91a2eafe84b9e301392f980))
* parse publish section in config ([721de6e](https://github.com/kaans/mqtli/commit/721de6e0f9bd45eb804300c0b0722fe8de01f240))
* **payload:** Add conversion from yaml to protobuf ([32a78e4](https://github.com/kaans/mqtli/commit/32a78e4968b615492c072a8028fe941e62c61c5e))
* **payload:** convert from json and yaml to protobuf and vice-versa ([32a78e4](https://github.com/kaans/mqtli/commit/32a78e4968b615492c072a8028fe941e62c61c5e))
* print config on debug log ([83eb066](https://github.com/kaans/mqtli/commit/83eb06619f2522c29dcaf3382e2f1c0f15003ecb))
* print protobuf as text ([b99694a](https://github.com/kaans/mqtli/commit/b99694a7e0246762e1d0e16b035961af0d79e66f))
* read output format from config file ([136fe28](https://github.com/kaans/mqtli/commit/136fe28ecb6d7ebe114e8fba20cff863224343ac))
* refactored subscribed topics to be a more generic configuration for topics ([ab21b92](https://github.com/kaans/mqtli/commit/ab21b92ed546c9c18b1d7ef4fa489239c3dba2a6))
* switched to rustls for tls connection ([5c9d6d9](https://github.com/kaans/mqtli/commit/5c9d6d9b113bfa7645023cc09a834be34d7a6f80))
* **trigger:** publish trigger based messages with times relative to the start of the program instead of cron based scheduling ([17b777f](https://github.com/kaans/mqtli/commit/17b777f63d747c2740a5ddeb9c6ee4a0b8129f77))
* use raw bytes in base64 ([d9d31c5](https://github.com/kaans/mqtli/commit/d9d31c55e4260a84672c61b4e15f1b13da59a58a))
* use raw bytes in hex ([66905e6](https://github.com/kaans/mqtli/commit/66905e692962ba5d3da0f144167a2f580d4c9588))
* use raw bytes in text ([63ecbd9](https://github.com/kaans/mqtli/commit/63ecbd91261c7c211608a194a351e670373cb2f9))


### Bug Fixes

* convert json payload correclty from yaml ([943897c](https://github.com/kaans/mqtli/commit/943897cb9170bd01274d7917bb600957297d8256))
* examples imports ([f98f1b7](https://github.com/kaans/mqtli/commit/f98f1b7488746202222798c30624fef0d996361b))
* lots of cleanups, simplifications and bugfixes for payload formats ([025362b](https://github.com/kaans/mqtli/commit/025362b4cb9481d7381887ad9ce92dfc1631533e))
* parse json and yaml from text,base64,hex ([05e3ac4](https://github.com/kaans/mqtli/commit/05e3ac4dfc7e382a18c633e8b4abf780818e498e))
* parse yaml and json directly from other types without content attribute ([a5887ff](https://github.com/kaans/mqtli/commit/a5887ff581c8a251e0840c88fa883abd55478676))
* require config file and set config.yaml in current directory as default ([c9a52b4](https://github.com/kaans/mqtli/commit/c9a52b46fc3434cc55d0b1304d9876241b848c2e))
* set default for overwrite value ([dda0d63](https://github.com/kaans/mqtli/commit/dda0d63c40d4f511d7a2dc7fbb2ae708126913da))
* tests in comments ([6c3cba3](https://github.com/kaans/mqtli/commit/6c3cba35406cf0a984594ec0e2dbf3eeb21e3498))
* **trigger:** respect initial delay when publishing messages ([17b777f](https://github.com/kaans/mqtli/commit/17b777f63d747c2740a5ddeb9c6ee4a0b8129f77))
* **trigger:** use count value to limit number of published messages ([17b777f](https://github.com/kaans/mqtli/commit/17b777f63d747c2740a5ddeb9c6ee4a0b8129f77))
* typo in readme ([ad65094](https://github.com/kaans/mqtli/commit/ad650948da47adcaa6a2e8e870a9e07a95ef3779))
* use convert function to convert to protobuf payload ([86ead4f](https://github.com/kaans/mqtli/commit/86ead4f1e48e66f84122f940ca35dfcd6557e8ec))
* use correct conversions between formats ([0f897db](https://github.com/kaans/mqtli/commit/0f897dbe60d74918b6f0dbe65a695fbedc6ecd7d))


### Miscellaneous Chores

* release 0.3.0 ([99e33ef](https://github.com/kaans/mqtli/commit/99e33efa1205bb0eb920c2b966e4a5da5815bfe3))
* release 0.4.0 ([70d0310](https://github.com/kaans/mqtli/commit/70d031015c0e0ceeb5d01bae5147a4d1b5bae457))

## [0.3.0](https://github.com/kaans/mqtli/compare/v0.2.0...v0.3.0) (2024-01-09)


### Bug Fixes

* typo in readme ([69b1a2a](https://github.com/kaans/mqtli/commit/69b1a2a89f1fa7a6a194e850be97a7cc3c1aa1dc))


### Miscellaneous Chores

* release 0.3.0 ([3bc6bfa](https://github.com/kaans/mqtli/commit/3bc6bfaf72094b387e82f54514341259c557557b))
