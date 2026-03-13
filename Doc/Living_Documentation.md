# Axiom2d Living Documentation

> Auto-generated from 696 test cases. Last updated: 2026-03-13.

<details>
<summary><strong>axiom2d</strong> (15 tests)</summary>

<blockquote>
<details>
<summary><strong>default_plugins</strong> (15 tests)</summary>

<blockquote>
<details>
<summary>When atlas inserted and frame runs, then upload atlas called</summary>

<code>crates\axiom2d\src\default_plugins.rs:294</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When atlas uploaded, then draw sprite also called same frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:320</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When audio feature on, then audio res is present</summary>

<code>crates\axiom2d\src\default_plugins.rs:383</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When audio feature on, then play sound buffer is present</summary>

<code>crates\axiom2d\src\default_plugins.rs:369</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When child of entity spawned, then children component created after frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:143</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has transform2d, then global transform set after frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:162</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has visible false, then effective visibility false after frame</summary>

<code>crates\axiom2d\src\default_plugins.rs:187</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When frame advanced with fake clock, then delta time is updated</summary>

<code>crates\axiom2d\src\default_plugins.rs:126</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key pressed and frame runs, then input state reflects key</summary>

<code>crates\axiom2d\src\default_plugins.rs:107</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key pressed and second frame runs, then just pressed is false</summary>

<code>crates\axiom2d\src\default_plugins.rs:433</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mouse button pressed and frame runs, then mouse state reflects button</summary>

<code>crates\axiom2d\src\default_plugins.rs:396</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mouse button pressed and two frames run, then just pressed cleared</summary>

<code>crates\axiom2d\src\default_plugins.rs:415</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When renderer injected and frame runs, then clear called</summary>

<code>crates\axiom2d\src\default_plugins.rs:208</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape entity exists and frame runs, then draw shape called</summary>

<code>crates\axiom2d\src\default_plugins.rs:262</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite entity exists and frame runs, then draw sprite called</summary>

<code>crates\axiom2d\src\default_plugins.rs:227</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>demo</strong> (41 tests)</summary>

<blockquote>
<details>
<summary><strong>tests</strong> (41 tests)</summary>

<blockquote>
<details>
<summary>When background shapes queried, then all additive blend</summary>

<code>crates\demo\src\main.rs:1214</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When background shapes queried, then at least one exists</summary>

<code>crates\demo\src\main.rs:1196</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When background shapes queried, then polygon clusters exist</summary>

<code>crates\demo\src\main.rs:1232</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bloom settings queried, then enabled</summary>

<code>crates\demo\src\main.rs:1074</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When delta time is zero, then rotation unchanged</summary>

<code>crates\demo\src\main.rs:736</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has no orbital speed, then rotation unchanged</summary>

<code>crates\demo\src\main.rs:760</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When multiple pivots, then each rotates at own speed</summary>

<code>crates\demo\src\main.rs:778</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no pan input, then camera position unchanged</summary>

<code>crates\demo\src\main.rs:478</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When opposite pan directions, then camera x unchanged</summary>

<code>crates\demo\src\main.rs:497</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When orbit and propagation run, then planet position on circle</summary>

<code>crates\demo\src\main.rs:956</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When orbit system runs, then rotation increments by speed times delta</summary>

<code>crates\demo\src\main.rs:699</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When orbit system runs twice, then rotation accumulates</summary>

<code>crates\demo\src\main.rs:717</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When pan down, then camera moves down</summary>

<code>crates\demo\src\main.rs:458</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When pan left, then camera moves left</summary>

<code>crates\demo\src\main.rs:418</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When pan right, then camera moves right</summary>

<code>crates\demo\src\main.rs:396</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When pan up, then camera moves up</summary>

<code>crates\demo\src\main.rs:438</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When planet shapes queried, then distinct colors</summary>

<code>crates\demo\src\main.rs:1129</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When planets queried, then each has shape</summary>

<code>crates\demo\src\main.rs:1103</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When post update systems run, then player entity gains global transform</summary>

<code>crates\demo\src\main.rs:819</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When render phase runs, then clear before camera before sprite</summary>

<code>crates\demo\src\main.rs:629</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then action map has all bindings</summary>

<code>crates\demo\src\main.rs:800</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then all celestial entities exist</summary>

<code>crates\demo\src\main.rs:663</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then all planets on world render layer</summary>

<code>crates\demo\src\main.rs:921</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then bloom settings exist</summary>

<code>crates\demo\src\main.rs:1061</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then camera2d entity exists</summary>

<code>crates\demo\src\main.rs:685</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then camera centered on sun</summary>

<code>crates\demo\src\main.rs:941</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then correct number of planet pivots</summary>

<code>crates\demo\src\main.rs:885</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then each pivot has one planet child</summary>

<code>crates\demo\src\main.rs:899</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then exactly one sun entity exists</summary>

<code>crates\demo\src\main.rs:856</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then moon pivots have orbital speed</summary>

<code>crates\demo\src\main.rs:1040</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then moons are grandchildren of orbit pivots</summary>

<code>crates\demo\src\main.rs:1011</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then moons exist with moon marker</summary>

<code>crates\demo\src\main.rs:997</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then sun has yellow color</summary>

<code>crates\demo\src\main.rs:870</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When setup called, then sun is circle shape</summary>

<code>crates\demo\src\main.rs:1088</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shapes queried, then sun plus planets are circles</summary>

<code>crates\demo\src\main.rs:1176</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite render system runs, then draw sprite called for player</summary>

<code>crates\demo\src\main.rs:596</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprites queried, then only moons remain</summary>

<code>crates\demo\src\main.rs:1162</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform propagation runs, then root entity gets global transform</summary>

<code>crates\demo\src\main.rs:576</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When zoom in pressed, then camera zoom increases</summary>

<code>crates\demo\src\main.rs:519</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When zoom out at minimum, then zoom does not go below floor</summary>

<code>crates\demo\src\main.rs:557</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When zoom out pressed, then camera zoom decreases</summary>

<code>crates\demo\src\main.rs:538</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_app</strong> (39 tests)</summary>

<blockquote>
<details>
<summary><strong>app</strong> (35 tests)</summary>

<blockquote>
<details>
<summary>When add plugin chained twice, then does not panic</summary>

<code>crates\engine_app\src\app.rs:257</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When add systems chained, then builder pattern works</summary>

<code>crates\engine_app\src\app.rs:437</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app created, then renderer res not yet in world</summary>

<code>crates\engine_app\src\app.rs:570</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app default called, then plugin count is zero</summary>

<code>crates\engine_app\src\app.rs:315</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app new called, then delta time resource is present</summary>

<code>crates\engine_app\src\app.rs:622</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app new called, then plugin count is zero</summary>

<code>crates\engine_app\src\app.rs:238</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app new called, then window size resource is present</summary>

<code>crates\engine_app\src\app.rs:595</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app receives keyboard press, then event pushed to buffer</summary>

<code>crates\engine_app\src\app.rs:668</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app receives keyboard release, then release event pushed to buffer</summary>

<code>crates\engine_app\src\app.rs:684</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When app receives unidentified physical key, then buffer remains empty</summary>

<code>crates\engine_app\src\app.rs:703</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor moved event received by app, then screen pos updated</summary>

<code>crates\engine_app\src\app.rs:722</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor moved without mouse state resource, then does not panic</summary>

<code>crates\engine_app\src\app.rs:737</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When handle redraw called, then pre update runs before update</summary>

<code>crates\engine_app\src\app.rs:635</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When handle redraw called, then present called via renderer res</summary>

<code>crates\engine_app\src\app.rs:343</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When handle redraw called twice, then system runs twice</summary>

<code>crates\engine_app\src\app.rs:382</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When handle redraw called without renderer res, then does not panic</summary>

<code>crates\engine_app\src\app.rs:359</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When handle resize called, then renderer resize is called</summary>

<code>crates\engine_app\src\app.rs:579</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When handle resize called, then window size resource is updated</summary>

<code>crates\engine_app\src\app.rs:605</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mouse button event received by app, then event pushed to buffer</summary>

<code>crates\engine_app\src\app.rs:746</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mouse button event received without buffer resource, then does not panic</summary>

<code>crates\engine_app\src\app.rs:768</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When new app created, then five schedules exist</summary>

<code>crates\engine_app\src\app.rs:447</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When one plugin added, then plugin count is one</summary>

<code>crates\engine_app\src\app.rs:266</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When plugin added, then build called exactly once</summary>

<code>crates\engine_app\src\app.rs:300</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When plugin calls add systems, then system runs during handle redraw</summary>

<code>crates\engine_app\src\app.rs:512</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When plugin inserts resource, then resource persists after build</summary>

<code>crates\engine_app\src\app.rs:533</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When render phase system uses renderer res, then draw calls precede present</summary>

<code>crates\engine_app\src\app.rs:472</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When resource inserted into app world, then value is readable</summary>

<code>crates\engine_app\src\app.rs:456</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scroll event received by app, then mouse state scroll delta accumulated</summary>

<code>crates\engine_app\src\app.rs:777</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set renderer called, then renderer res present in world</summary>

<code>crates\engine_app\src\app.rs:556</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set window config called, then config is stored</summary>

<code>crates\engine_app\src\app.rs:324</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set window config called, then window size reflects config</summary>

<code>crates\engine_app\src\app.rs:792</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system added to update phase, then runs during handle redraw</summary>

<code>crates\engine_app\src\app.rs:368</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When systems in all phases, then run in canonical order</summary>

<code>crates\engine_app\src\app.rs:397</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two distinct plugins added, then plugin count is two</summary>

<code>crates\engine_app\src\app.rs:278</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When update systems exist, then schedules run and present called</summary>

<code>crates\engine_app\src\app.rs:493</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>mouse_world_pos_system</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>When world pos system runs with camera, then world pos is screen to world converted</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:54</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When world pos system runs with no camera, then uses default camera</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:68</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When world pos system runs with offset cursor and zoom, then world pos is scaled</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:98</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When world pos system runs with zoomed camera, then center still maps to camera pos</summary>

<code>crates\engine_app\src\mouse_world_pos_system.rs:81</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_assets</strong> (26 tests)</summary>

<blockquote>
<details>
<summary><strong>asset_server</strong> (14 tests)</summary>

<blockquote>
<details>
<summary>When adding asset, then returns handle with id zero</summary>

<code>crates\engine_assets\src\asset_server.rs:92</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding second asset, then returns different handle</summary>

<code>crates\engine_assets\src\asset_server.rs:104</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When asset added, then ref count is one</summary>

<code>crates\engine_assets\src\asset_server.rs:159</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When clone handle called, then ref count increments</summary>

<code>crates\engine_assets\src\asset_server.rs:171</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cloned n times, then ref count lifecycle is correct</summary>

<code>crates\engine_assets\src\asset_server.rs:229</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When getting by handle, then returns stored value</summary>

<code>crates\engine_assets\src\asset_server.rs:118</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When getting mut, then mutation is visible on next get</summary>

<code>crates\engine_assets\src\asset_server.rs:144</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When getting unknown handle, then returns none</summary>

<code>crates\engine_assets\src\asset_server.rs:131</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading invalid ron, then returns parse error</summary>

<code>crates\engine_assets\src\asset_server.rs:275</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading nonexistent file, then returns io error</summary>

<code>crates\engine_assets\src\asset_server.rs:263</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading valid ron file, then returns handle to deserialized value</summary>

<code>crates\engine_assets\src\asset_server.rs:292</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When remove unknown handle, then returns false</summary>

<code>crates\engine_assets\src\asset_server.rs:215</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When remove with ref count above one, then decrements without evicting</summary>

<code>crates\engine_assets\src\asset_server.rs:184</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When remove with ref count one, then evicts asset</summary>

<code>crates\engine_assets\src\asset_server.rs:200</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>scene</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>When invalid ron deserialized as scene def, then returns error</summary>

<code>crates\engine_assets\src\scene.rs:141</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When minimal scene def serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:153</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene def with all component types roundtrips, then all fields survive</summary>

<code>crates\engine_assets\src\scene.rs:325</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene def with parent child serialized, then parent index is preserved</summary>

<code>crates\engine_assets\src\scene.rs:118</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene node def serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_assets\src\scene.rs:61</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene node def with all fields serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:173</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene node with none sprite serialized, then roundtrips as none</summary>

<code>crates\engine_assets\src\scene.rs:84</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene node with some sprite serialized, then roundtrips with matching fields</summary>

<code>crates\engine_assets\src\scene.rs:97</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene with audio emitter serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:276</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene with convex polygon collider serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:220</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene with material serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:295</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scene with shape variants serialized to pretty ron, then snapshot matches</summary>

<code>crates\engine_assets\src\scene.rs:241</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_audio</strong> (63 tests)</summary>

<blockquote>
<details>
<summary><strong>audio_backend</strong> (6 tests)</summary>

<blockquote>
<details>
<summary>When play called, then returns playback id</summary>

<code>crates\engine_audio\src\audio_backend.rs:66</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play called twice, then ids differ</summary>

<code>crates\engine_audio\src\audio_backend.rs:76</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play on track called, then returns playback id</summary>

<code>crates\engine_audio\src\audio_backend.rs:120</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set track volume on null backend, then no panic</summary>

<code>crates\engine_audio\src\audio_backend.rs:110</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set volume called, then does not panic</summary>

<code>crates\engine_audio\src\audio_backend.rs:99</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When stop called, then does not panic</summary>

<code>crates\engine_audio\src\audio_backend.rs:90</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>cpal_backend</strong> (14 tests)</summary>

<blockquote>
<details>
<summary>When constructed, then volume is one</summary>

<code>crates\engine_audio\src\cpal_backend.rs:229</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When global and track volume both half, then output quarter</summary>

<code>crates\engine_audio\src\cpal_backend.rs:386</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mix into with two tracks, then per track volume applied</summary>

<code>crates\engine_audio\src\cpal_backend.rs:364</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play called, then active sound added</summary>

<code>crates\engine_audio\src\cpal_backend.rs:261</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play called twice, then ids are unique</summary>

<code>crates\engine_audio\src\cpal_backend.rs:238</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set track volume on cpal, then internal state updated</summary>

<code>crates\engine_audio\src\cpal_backend.rs:405</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When single sound at full volume, then output matches samples</summary>

<code>crates\engine_audio\src\cpal_backend.rs:301</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When single sound at half volume, then output is scaled</summary>

<code>crates\engine_audio\src\cpal_backend.rs:316</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sound longer than buffer, then cursor advances</summary>

<code>crates\engine_audio\src\cpal_backend.rs:418</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sound shorter than buffer, then removed after last sample</summary>

<code>crates\engine_audio\src\cpal_backend.rs:347</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When stop called, then sound removed from active list</summary>

<code>crates\engine_audio\src\cpal_backend.rs:274</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When stop with unknown id, then does not panic</summary>

<code>crates\engine_audio\src\cpal_backend.rs:252</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two active sounds, then output is sum</summary>

<code>crates\engine_audio\src\cpal_backend.rs:330</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sounds and stop one, then other remains</summary>

<code>crates\engine_audio\src\cpal_backend.rs:287</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>mixer</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>When default mixer state, then all tracks are one</summary>

<code>crates\engine_audio\src\mixer.rs:69</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mixer track variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_audio\src\mixer.rs:60</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set track volume, then only that track changes</summary>

<code>crates\engine_audio\src\mixer.rs:83</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When volume above one, then stored unchanged</summary>

<code>crates\engine_audio\src\mixer.rs:97</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>play_sound_buffer</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>When drained, then buffer is empty</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:105</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play sound new, then track defaults to sfx</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:73</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play sound on track, then track is preserved</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:82</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When push and drain, then returns one command</summary>

<code>crates\engine_audio\src\play_sound_buffer.rs:91</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>play_sound_system</strong> (9 tests)</summary>

<blockquote>
<details>
<summary>When both gains zero, then play sound skips backend</summary>

<code>crates\engine_audio\src\play_sound_system.rs:337</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When known sound, then audio play is called</summary>

<code>crates\engine_audio\src\play_sound_system.rs:163</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When known sound, then buffer is drained</summary>

<code>crates\engine_audio\src\play_sound_system.rs:182</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no sound library, then audio not called</summary>

<code>crates\engine_audio\src\play_sound_system.rs:202</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play sound with mixer state, then track volume forwarded</summary>

<code>crates\engine_audio\src\play_sound_system.rs:238</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play sound without mixer state, then runs normally</summary>

<code>crates\engine_audio\src\play_sound_system.rs:275</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When spatial gains present, then play sound applies them</summary>

<code>crates\engine_audio\src\play_sound_system.rs:315</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When unknown sound name, then audio not called</summary>

<code>crates\engine_audio\src\play_sound_system.rs:219</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When unknown sound name, then buffer is still drained</summary>

<code>crates\engine_audio\src\play_sound_system.rs:295</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>sound_data</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When mono, then frame count equals sample len</summary>

<code>crates\engine_audio\src\sound_data.rs:21</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When stereo, then frame count is half sample len</summary>

<code>crates\engine_audio\src\sound_data.rs:37</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>sound_effect</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>When nonzero amplitude graph, then samples are not all zero</summary>

<code>crates\engine_audio\src\sound_effect.rs:87</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When synthesize called, then sample length equals frame count times channels</summary>

<code>crates\engine_audio\src\sound_effect.rs:75</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When synthesize called, then sound data has correct sample rate</summary>

<code>crates\engine_audio\src\sound_effect.rs:51</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When synthesize called, then sound data has mono channel count</summary>

<code>crates\engine_audio\src\sound_effect.rs:63</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When synthesize called twice, then each call returns fresh sound data</summary>

<code>crates\engine_audio\src\sound_effect.rs:99</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>sound_library</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>When empty library, then get returns none</summary>

<code>crates\engine_audio\src\sound_library.rs:35</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When registered, then get with different name returns none</summary>

<code>crates\engine_audio\src\sound_library.rs:60</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When registered, then get with same name returns some</summary>

<code>crates\engine_audio\src\sound_library.rs:47</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>spatial</strong> (16 tests)</summary>

<blockquote>
<details>
<summary>When any distance, then attenuation in zero to one</summary>

<code>crates\engine_audio\src\spatial.rs:274</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When any two positions, then constant power property holds</summary>

<code>crates\engine_audio\src\spatial.rs:292</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When audio emitter serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_audio\src\spatial.rs:119</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When distance equals max, then attenuation is zero</summary>

<code>crates\engine_audio\src\spatial.rs:183</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When distance exceeds max, then attenuation clamped to zero</summary>

<code>crates\engine_audio\src\spatial.rs:201</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When distance half max, then attenuation is half</summary>

<code>crates\engine_audio\src\spatial.rs:192</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When distance zero, then attenuation is one</summary>

<code>crates\engine_audio\src\spatial.rs:174</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When emitter ahead of listener, then gains equal</summary>

*Centered panning when emitter is on listener's forward axis — no left/right bias*

<code>crates\engine_audio\src\spatial.rs:236</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When emitter at listener, then gains equal no nan</summary>

*Coincident positions must not produce NaN — atan2(0,0) edge case handled by defaulting to centered pan*

<code>crates\engine_audio\src\spatial.rs:254</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When emitter beyond max distance, then gains are zero</summary>

*Linear distance attenuation drops to zero beyond max_distance, effectively culling inaudible sounds*

<code>crates\engine_audio\src\spatial.rs:366</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When emitter is child entity, then world position used</summary>

*Spatial audio uses GlobalTransform2D (world space), not local Transform2D — hierarchy must propagate first*

<code>crates\engine_audio\src\spatial.rs:405</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When emitter left of listener, then left gain one</summary>

<code>crates\engine_audio\src\spatial.rs:225</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When emitter right of listener, then right gain one</summary>

*Constant-power stereo panning — emitter fully to the right produces 100% right channel gain*

<code>crates\engine_audio\src\spatial.rs:212</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When emitter to right, then spatial gains reflect pan and attenuation</summary>

<code>crates\engine_audio\src\spatial.rs:315</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no listener, then system runs without panic</summary>

*Without an AudioListener entity, spatial processing is a no-op — gains remain unchanged*

<code>crates\engine_audio\src\spatial.rs:347</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When play sound without emitter, then gains unchanged</summary>

<code>crates\engine_audio\src\spatial.rs:386</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_core</strong> (39 tests)</summary>

<blockquote>
<details>
<summary><strong>color</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>When any finite color, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\color.rs:126</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When any u8 inputs, then from u8 components in zero to one</summary>

<code>crates\engine_core\src\color.rs:109</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When color from u8 called, then converts to normalized f32</summary>

<code>crates\engine_core\src\color.rs:96</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When color serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_core\src\color.rs:70</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transparent color serialized to ron, then roundtrip preserves zero alpha</summary>

<code>crates\engine_core\src\color.rs:83</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>error</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>When engine error boxed, then implements std error</summary>

<code>crates\engine_core\src\error.rs:40</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When engine error debug formatted, then identifies variant</summary>

<code>crates\engine_core\src\error.rs:46</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When invalid input displayed, then contains reason</summary>

<code>crates\engine_core\src\error.rs:28</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When not found displayed, then contains resource name</summary>

<code>crates\engine_core\src\error.rs:16</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>time</strong> (13 tests)</summary>

<blockquote>
<details>
<summary>When any positive delta and step size, then accumulator stays below step size</summary>

<code>crates\engine_core\src\time.rs:294</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When clock res derefmut, then reaches inner delta</summary>

<code>crates\engine_core\src\time.rs:204</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When fake clock advanced multiple times, then delta accumulates</summary>

<code>crates\engine_core\src\time.rs:175</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When fake clock advanced, then delta returns advancement</summary>

*FakeClock enables deterministic testing — advance() accumulates, delta() drains*

<code>crates\engine_core\src\time.rs:149</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When fake clock behind dyn time, then delta is correct</summary>

<code>crates\engine_core\src\time.rs:190</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When fake clock constructed, then delta is zero</summary>

<code>crates\engine_core\src\time.rs:139</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When fake clock delta called twice, then second call returns zero</summary>

<code>crates\engine_core\src\time.rs:161</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tick across frames, then accumulator carries forward</summary>

*Accumulator carries sub-step remainder across frames, ensuring no simulation time is lost*

<code>crates\engine_core\src\time.rs:260</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tick below step size, then returns zero steps</summary>

*Sub-step deltas accumulate silently — no simulation steps fire until a full step_size is reached*

<code>crates\engine_core\src\time.rs:219</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tick exactly one step, then returns one step</summary>

<code>crates\engine_core\src\time.rs:231</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tick large delta, then returns multiple steps and retains remainder</summary>

*Fix Your Timestep pattern — large frame deltas produce multiple fixed steps with leftover accumulated for the next frame*

<code>crates\engine_core\src\time.rs:245</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When time system runs, then delta time updated from clock</summary>

<code>crates\engine_core\src\time.rs:274</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When time system runs twice without advance, then second delta is zero</summary>

<code>crates\engine_core\src\time.rs:320</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>transform</strong> (11 tests)</summary>

<blockquote>
<details>
<summary>When any finite transform2d, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\transform.rs:177</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When default transform converted to affine2, then equals identity</summary>

<code>crates\engine_core\src\transform.rs:68</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform2d serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_core\src\transform.rs:35</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform2d with negative rotation serialized to ron, then roundtrip preserves sign</summary>

<code>crates\engine_core\src\transform.rs:52</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform composed, then order is scale rotate translate</summary>

<code>crates\engine_core\src\transform.rs:143</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform has all components, then affine2 matches scale angle translation</summary>

<code>crates\engine_core\src\transform.rs:122</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform has full circle rotation, then affine2 is near identity</summary>

<code>crates\engine_core\src\transform.rs:201</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform has negative scale, then affine2 preserves flip</summary>

<code>crates\engine_core\src\transform.rs:161</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform has rotation only, then affine2 is pure rotation</summary>

<code>crates\engine_core\src\transform.rs:92</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform has scale only, then affine2 is pure scale</summary>

<code>crates\engine_core\src\transform.rs:107</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform has translation only, then affine2 is pure translation</summary>

<code>crates\engine_core\src\transform.rs:77</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>types</strong> (6 tests)</summary>

<blockquote>
<details>
<summary>When any finite pixels, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\types.rs:115</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When any finite seconds, then ron roundtrip preserves value</summary>

<code>crates\engine_core\src\types.rs:128</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When negative pixels serialized to ron, then roundtrip preserves sign</summary>

<code>crates\engine_core\src\types.rs:87</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When newtypes serialized to ron, then deserialize to equal value</summary>

<code>crates\engine_core\src\types.rs:65</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When pixels arithmetic, then add sub mul produce correct results</summary>

<code>crates\engine_core\src\types.rs:100</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When seconds arithmetic, then add sub mul produce correct results</summary>

<code>crates\engine_core\src\types.rs:107</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_ecs</strong> (1 tests)</summary>

<blockquote>
<details>
<summary><strong>schedule</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>When index, then matches declaration order</summary>

<code>crates\engine_ecs\src\schedule.rs:34</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_input</strong> (53 tests)</summary>

<blockquote>
<details>
<summary><strong>action_map</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When multiple keys bound to same action, then all keys returned</summary>

<code>crates\engine_input\src\action_map.rs:36</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When single key bound to action, then bindings for returns that key</summary>

<code>crates\engine_input\src\action_map.rs:51</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>input_event_buffer</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When buffer drained, then returns all events and buffer is empty</summary>

<code>crates\engine_input\src\input_event_buffer.rs:42</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key event pushed, then drain returns one event</summary>

<code>crates\engine_input\src\input_event_buffer.rs:30</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>input_state</strong> (17 tests)</summary>

<blockquote>
<details>
<summary>When action not in map, then action just pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:205</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When action not in map, then action pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:162</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bound key held across frame clear, then action just pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:234</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bound key is just pressed, then action just pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:219</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bound key is not pressed, then action pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:176</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bound key is pressed, then action pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:190</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When frame cleared, then held key stays pressed</summary>

<code>crates\engine_input\src\input_state.rs:280</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When frame cleared, then just pressed is false for held key</summary>

<code>crates\engine_input\src\input_state.rs:135</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When frame cleared, then just released is false</summary>

<code>crates\engine_input\src\mouse_state.rs:177</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When input state default, then no keys are pressed</summary>

<code>crates\engine_input\src\input_state.rs:61</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key pressed, then just pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:85</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key pressed, then just released returns false</summary>

<code>crates\engine_input\src\input_state.rs:97</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key pressed, then pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:73</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key released after press, then just released returns true</summary>

<code>crates\engine_input\src\input_state.rs:122</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When key released after press, then pressed returns false</summary>

<code>crates\engine_input\src\input_state.rs:109</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When one of multiple bound keys is just pressed, then action just pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:250</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When one of multiple bound keys is pressed, then action pressed returns true</summary>

<code>crates\engine_input\src\input_state.rs:265</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>input_system</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>When press event in buffer, then key is just pressed</summary>

<code>crates\engine_input\src\input_system.rs:58</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When press event in buffer, then key is pressed</summary>

<code>crates\engine_input\src\input_system.rs:43</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When release event in buffer, then key is just released</summary>

<code>crates\engine_input\src\input_system.rs:94</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When release event in buffer, then key is not pressed</summary>

<code>crates\engine_input\src\input_system.rs:77</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs second frame, then just pressed is cleared</summary>

<code>crates\engine_input\src\input_system.rs:126</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs second frame, then just released is cleared</summary>

<code>crates\engine_input\src\input_system.rs:144</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs, then buffer is drained</summary>

<code>crates\engine_input\src\input_system.rs:111</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>mouse_event_buffer</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When buffer drained, then buffer is empty on second drain</summary>

<code>crates\engine_input\src\mouse_event_buffer.rs:44</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button event pushed, then drain returns that event</summary>

<code>crates\engine_input\src\mouse_event_buffer.rs:30</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>mouse_input_system</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>When mouse input system runs second frame, then just pressed is cleared</summary>

<code>crates\engine_input\src\mouse_input_system.rs:112</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mouse input system runs, then buffer is drained</summary>

<code>crates\engine_input\src\mouse_input_system.rs:97</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When press event in buffer, then mouse input system sets button pressed</summary>

<code>crates\engine_input\src\mouse_input_system.rs:44</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When press event in buffer, then mouse input system sets just pressed</summary>

<code>crates\engine_input\src\mouse_input_system.rs:59</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When release event in buffer, then mouse input system sets just released</summary>

<code>crates\engine_input\src\mouse_input_system.rs:78</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>mouse_state</strong> (18 tests)</summary>

<blockquote>
<details>
<summary>When action bound to mouse button and button just pressed, then action just pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:296</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When action bound to mouse button and button not pressed, then action pressed returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:281</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When action bound to mouse button and button pressed, then action pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:266</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button pressed, then just pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:113</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button pressed, then just released returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:125</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button pressed, then pressed returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:101</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button released after press, then just released returns true</summary>

<code>crates\engine_input\src\mouse_state.rs:150</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button released after press, then pressed returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:137</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor moved, then screen pos is updated</summary>

<code>crates\engine_input\src\mouse_state.rs:191</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When frame cleared, then just pressed is false for held button</summary>

<code>crates\engine_input\src\mouse_state.rs:163</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When frame cleared, then just released is false</summary>

<code>crates\engine_input\src\mouse_state.rs:177</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When frame cleared, then scroll delta is zero</summary>

<code>crates\engine_input\src\mouse_state.rs:215</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When multiple scroll events in one frame, then delta is sum</summary>

<code>crates\engine_input\src\mouse_state.rs:228</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no buttons pressed, then mouse state reports nothing pressed</summary>

<code>crates\engine_input\src\mouse_state.rs:90</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When screen pos set, then clear frame state does not reset it</summary>

<code>crates\engine_input\src\mouse_state.rs:241</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When scroll accumulated, then scroll delta reflects total</summary>

<code>crates\engine_input\src\mouse_state.rs:203</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When unbound mouse action queried, then action pressed returns false</summary>

<code>crates\engine_input\src\mouse_state.rs:312</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When world pos set, then world pos is readable</summary>

<code>crates\engine_input\src\mouse_state.rs:254</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_physics</strong> (42 tests)</summary>

<blockquote>
<details>
<summary><strong>collider</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When collider variants serialized to ron, then each deserializes to equal value</summary>

<code>crates\engine_physics\src\collider.rs:36</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When convex polygon collider debug formatted, then snapshot matches</summary>

<code>crates\engine_physics\src\collider.rs:18</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>collision_event</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>When drained twice, then second drain returns empty</summary>

<code>crates\engine_physics\src\collision_event.rs:72</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When empty buffer drained, then returns empty iterator</summary>

<code>crates\engine_physics\src\collision_event.rs:40</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When event pushed and drained, then yields that event</summary>

<code>crates\engine_physics\src\collision_event.rs:52</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>physics_backend</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>When add body, then returns true and duplicate returns false</summary>

<code>crates\engine_physics\src\physics_backend.rs:90</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When add collider, then returns true</summary>

<code>crates\engine_physics\src\physics_backend.rs:157</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When body position queried for unregistered, then returns none</summary>

<code>crates\engine_physics\src\physics_backend.rs:105</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null backend drain collision events, then returns empty</summary>

<code>crates\engine_physics\src\physics_backend.rs:145</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When remove body for unknown entity, then no panic</summary>

<code>crates\engine_physics\src\physics_backend.rs:135</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When remove body, then entity is deregistered</summary>

<code>crates\engine_physics\src\physics_backend.rs:120</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When step called, then step count increments</summary>

<code>crates\engine_physics\src\physics_backend.rs:76</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>physics_step_system</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>When backend produces events, then buffer contains them</summary>

<code>crates\engine_physics\src\physics_step_system.rs:143</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs, then backend is stepped</summary>

<code>crates\engine_physics\src\physics_step_system.rs:95</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs twice, then backend stepped twice</summary>

<code>crates\engine_physics\src\physics_step_system.rs:127</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs with no events, then buffer remains empty</summary>

<code>crates\engine_physics\src\physics_step_system.rs:110</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>physics_sync_system</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>When backend returns both position and rotation, then both fields updated</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:152</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When backend returns none for unregistered entity, then transform is unchanged</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:174</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When backend returns position only, then rotation field is unchanged</summary>

*Position and rotation are synced independently — either can be None without affecting the other*

<code>crates\engine_physics\src\physics_sync_system.rs:200</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When backend returns position, then transform position is updated</summary>

*One-way sync: physics backend → Transform2D. ECS is the read side, rapier is the authority*

<code>crates\engine_physics\src\physics_sync_system.rs:110</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When backend returns rotation only, then position field is unchanged</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:227</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When backend returns rotation, then transform rotation is updated</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:131</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has no rigid body, then its transform is not touched</summary>

*Only entities with RigidBody participate in physics sync — plain transforms are untouched*

<code>crates\engine_physics\src\physics_sync_system.rs:309</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When multiple entities registered, then each entity receives its own transform</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:280</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no entities have rigid body, then system runs without panic</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:99</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rigid body is kinematic, then transform is still synced</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:355</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rigid body is static, then transform is still synced</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:335</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs, then transform scale is never modified</summary>

<code>crates\engine_physics\src\physics_sync_system.rs:253</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>rapier_backend</strong> (13 tests)</summary>

<blockquote>
<details>
<summary>When add collider for unknown entity, then returns false</summary>

<code>crates\engine_physics\src\rapier_backend.rs:270</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When body removed after collision, then drain does not panic</summary>

<code>crates\engine_physics\src\rapier_backend.rs:374</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When body type mapping, then static is fixed and kinematic is position based</summary>

*Body type mapping: ECS Static → rapier Fixed (immovable), ECS Kinematic → rapier KinematicPositionBased (script-driven)*

<code>crates\engine_physics\src\rapier_backend.rs:219</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When collider variants added, then all return true</summary>

<code>crates\engine_physics\src\rapier_backend.rs:246</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When drain called twice without step, then second is empty</summary>

<code>crates\engine_physics\src\rapier_backend.rs:355</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When dynamic body added, then position is queryable</summary>

*Body type mapping: ECS Dynamic → rapier Dynamic (free motion under forces)*

<code>crates\engine_physics\src\rapier_backend.rs:187</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When dynamic body steps under gravity, then y changes</summary>

<code>crates\engine_physics\src\rapier_backend.rs:283</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no colliders step and drain, then no events</summary>

<code>crates\engine_physics\src\rapier_backend.rs:315</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rapier step on empty world, then no panic</summary>

<code>crates\engine_physics\src\rapier_backend.rs:177</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When remove body for unknown entity on rapier, then no panic</summary>

<code>crates\engine_physics\src\rapier_backend.rs:394</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When remove body on rapier, then position returns none</summary>

*Entity removal must clean up both rapier RigidBody and the entity↔handle map*

<code>crates\engine_physics\src\rapier_backend.rs:300</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When same entity added twice, then second returns false</summary>

<code>crates\engine_physics\src\rapier_backend.rs:204</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two overlapping circles step, then started event with correct entities</summary>

*Collision events flow: rapier ChannelEventCollector → drain → CollisionEventBuffer with entity resolution*

<code>crates\engine_physics\src\rapier_backend.rs:329</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>rigid_body</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>When rigid body variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_physics\src\rigid_body.rs:17</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_render</strong> (234 tests)</summary>

<blockquote>
<details>
<summary><strong>atlas</strong> (31 tests)</summary>

<blockquote>
<details>
<summary>When adding image larger than atlas, then returns no space error</summary>

<code>crates\engine_render\src\atlas.rs:344</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding image that fills atlas, then uv rect is full range</summary>

<code>crates\engine_render\src\atlas.rs:277</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding image, then uv rect is normalized to zero one</summary>

<code>crates\engine_render\src\atlas.rs:259</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding many images, then all uv rects are non overlapping</summary>

<code>crates\engine_render\src\atlas.rs:322</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding single image, then returns handle with valid texture id</summary>

<code>crates\engine_render\src\atlas.rs:247</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding two images, then each has distinct texture id</summary>

<code>crates\engine_render\src\atlas.rs:289</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding two images, then uv rects do not overlap</summary>

<code>crates\engine_render\src\atlas.rs:302</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding zero height image, then returns invalid dimensions</summary>

<code>crates\engine_render\src\atlas.rs:399</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When adding zero width image, then returns invalid dimensions</summary>

<code>crates\engine_render\src\atlas.rs:387</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When atlas full, then returns no space error</summary>

<code>crates\engine_render\src\atlas.rs:356</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When atlas present, then upload atlas called</summary>

<code>crates\engine_render\src\atlas.rs:669</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When atlas uploaded marker present, then upload atlas not called</summary>

<code>crates\engine_render\src\atlas.rs:720</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When builder created, then reports matching dimensions</summary>

<code>crates\engine_render\src\atlas.rs:233</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When building atlas, then all rows of image are correctly placed</summary>

<code>crates\engine_render\src\atlas.rs:568</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When building atlas, then pixel data appears at correct offset</summary>

<code>crates\engine_render\src\atlas.rs:456</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When building atlas with image, then buffer size matches atlas</summary>

<code>crates\engine_render\src\atlas.rs:219</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When building atlas with two images, then neither overwrites the other</summary>

<code>crates\engine_render\src\atlas.rs:474</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When building empty atlas, then pixel buffer is all zeros</summary>

<code>crates\engine_render\src\atlas.rs:207</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When data length mismatches, then returns error</summary>

<code>crates\engine_render\src\atlas.rs:369</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading image and adding to atlas, then dimensions preserved</summary>

<code>crates\engine_render\src\atlas.rs:532</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading invalid bytes, then returns decode error</summary>

<code>crates\engine_render\src\atlas.rs:523</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading valid png, then returns correct image data</summary>

<code>crates\engine_render\src\atlas.rs:509</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When looking up known texture id, then returns matching uv rect</summary>

<code>crates\engine_render\src\atlas.rs:411</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When looking up multiple textures, then each returns its own uv rect</summary>

<code>crates\engine_render\src\atlas.rs:439</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When looking up unknown texture id, then returns none</summary>

<code>crates\engine_render\src\atlas.rs:425</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no atlas, then upload atlas not called</summary>

<code>crates\engine_render\src\atlas.rs:683</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When normalizing uv rect at origin, then starts at zero</summary>

<code>crates\engine_render\src\atlas.rs:559</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When normalizing uv rect, then output is in zero one range</summary>

<code>crates\engine_render\src\atlas.rs:550</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When second image at nonzero y, then uv height matches image ratio</summary>

<code>crates\engine_render\src\atlas.rs:603</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When second image offset, then handle uv matches build lookup</summary>

<code>crates\engine_render\src\atlas.rs:621</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs twice, then upload atlas called only once</summary>

<code>crates\engine_render\src\atlas.rs:696</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>bloom</strong> (9 tests)</summary>

- System tests::when bloom disabled then post process system skips
- System tests::when no bloom settings then post process system skips
- System tests::when post process system runs then log records apply post process
<blockquote>
<details>
<summary>When any radius, then gaussian weights sum to one and are symmetric</summary>

<code>crates\engine_render\src\bloom.rs:193</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When gaussian weights computed, then kernel is symmetric</summary>

<code>crates\engine_render\src\bloom.rs:81</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When gaussian weights radius0, then single weight of one</summary>

<code>crates\engine_render\src\bloom.rs:99</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When gaussian weights radius1, then center is largest</summary>

<code>crates\engine_render\src\bloom.rs:56</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When gaussian weights radius3, then sum is one</summary>

<code>crates\engine_render\src\bloom.rs:67</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When gaussian weights radius3, then weight ratios match formula</summary>

<code>crates\engine_render\src\bloom.rs:235</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>camera</strong> (31 tests)</summary>

<blockquote>
<details>
<summary>When any world point, then screen to world of world to screen recovers original</summary>

<code>crates\engine_render\src\camera.rs:319</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera2d created with defaults, then position is zero and zoom is one</summary>

<code>crates\engine_render\src\camera.rs:144</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera2d serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\camera.rs:128</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera at nonzero position, then view matrix translates by negative position</summary>

<code>crates\engine_render\src\camera.rs:166</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera at origin with zoom half, then view matrix scales by half</summary>

<code>crates\engine_render\src\camera.rs:199</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera at origin with zoom one, then view matrix is identity</summary>

<code>crates\engine_render\src\camera.rs:154</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera at origin with zoom two, then view matrix scales by two</summary>

<code>crates\engine_render\src\camera.rs:183</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera at position with nonunit zoom, then view matrix combines translation and scale</summary>

<code>crates\engine_render\src\camera.rs:215</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera prepare system runs with camera, then set view projection called</summary>

<code>crates\engine_render\src\camera.rs:532</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera prepare system runs without camera, then default ortho set</summary>

*camera_prepare_system always sets a projection — defaults to viewport-centered ortho when no Camera2D entity exists*

<code>crates\engine_render\src\camera.rs:548</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera uniform from camera at center, then viewport corners map to ndc corners</summary>

<code>crates\engine_render\src\camera.rs:511</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera uniform from camera at origin zoom one, then origin maps to ndc center</summary>

*Default camera produces pixel-perfect 1:1 mapping — world origin lands at NDC center*

<code>crates\engine_render\src\camera.rs:479</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When camera uniform y flip, then top maps to positive ndc y</summary>

<code>crates\engine_render\src\camera.rs:562</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity completely above view, then aabb intersects returns false</summary>

<code>crates\engine_render\src\camera.rs:423</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity completely below view, then aabb intersects returns false</summary>

<code>crates\engine_render\src\camera.rs:434</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity completely left of view, then aabb intersects returns false</summary>

*Frustum culling AABB test — entity fully outside on any axis means no intersection*

<code>crates\engine_render\src\camera.rs:401</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity completely right of view, then aabb intersects returns false</summary>

<code>crates\engine_render\src\camera.rs:412</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity contains entire view, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:467</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity exactly touches view edge, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:456</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity fully inside view, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:389</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity partially overlaps left edge, then aabb intersects returns true</summary>

<code>crates\engine_render\src\camera.rs:445</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no camera, then system uses viewport center</summary>

<code>crates\engine_render\src\camera.rs:626</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When screen center, then screen to world returns camera position</summary>

<code>crates\engine_render\src\camera.rs:283</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When screen to world after world to screen, then recovers original point</summary>

*world_to_screen and screen_to_world are exact inverses — roundtrip recovers the original point*

<code>crates\engine_render\src\camera.rs:300</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When view rect at zoom one, then half extents equal half viewport</summary>

<code>crates\engine_render\src\camera.rs:353</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When view rect at zoom two, then half extents are halved</summary>

<code>crates\engine_render\src\camera.rs:371</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When viewport height zero, then camera prepare system skips</summary>

<code>crates\engine_render\src\camera.rs:608</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When viewport width zero, then camera prepare system skips</summary>

<code>crates\engine_render\src\camera.rs:590</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When world point at viewport corner, then world to screen returns corner</summary>

<code>crates\engine_render\src\camera.rs:250</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When world point at zoom two, then world to screen reflects magnification</summary>

*Zoom multiplies screen-space distances — zoom 2 means objects appear 2x larger*

<code>crates\engine_render\src\camera.rs:267</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When world point matches camera center, then world to screen returns screen center</summary>

*Camera position defines the world point that appears at screen center*

<code>crates\engine_render\src\camera.rs:234</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>clear</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>When clear system runs, then renderer clear receives clear color value</summary>

<code>crates\engine_render\src\clear.rs:31</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>material</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>When blend mode variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_render\src\material.rs:176</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When comparing blend modes, then alpha less than additive less than multiply</summary>

<code>crates\engine_render\src\material.rs:185</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When comparing shader handles, then ordered by inner u32</summary>

<code>crates\engine_render\src\material.rs:297</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When effective shader handle with none, then returns default</summary>

<code>crates\engine_render\src\material.rs:273</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When effective shader handle with some, then returns material shader</summary>

<code>crates\engine_render\src\material.rs:282</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When looking up unregistered handle, then returns none</summary>

<code>crates\engine_render\src\material.rs:224</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When material2d with textures and uniforms debug formatted, then snapshot matches</summary>

<code>crates\engine_render\src\material.rs:150</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When preprocessing with define present, then ifdef block included</summary>

<code>crates\engine_render\src\material.rs:236</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When preprocessing without define, then ifdef block excluded</summary>

<code>crates\engine_render\src\material.rs:309</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When registering multiple shaders, then handles are unique</summary>

<code>crates\engine_render\src\material.rs:211</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When registering shader, then lookup returns same source</summary>

<code>crates\engine_render\src\material.rs:197</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shader registry used as resource in system, then lookup works</summary>

<code>crates\engine_render\src\material.rs:252</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>rect</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When rect has negative pixel values, then stores without clamping</summary>

<code>crates\engine_render\src\rect.rs:54</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rect serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\rect.rs:35</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>renderer</strong> (13 tests)</summary>

<blockquote>
<details>
<summary>When null renderer applies post process, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:197</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer clears, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:92</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer draws rect, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:110</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer draws shape, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:128</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer draws sprite, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:119</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer presents, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:101</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer resizes, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:148</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer set blend mode, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:175</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer set shader and uniforms and texture, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:186</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer set view projection, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:139</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer upload atlas, then does not panic</summary>

<code>crates\engine_render\src\renderer.rs:166</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When null renderer viewport size, then returns zero zero</summary>

<code>crates\engine_render\src\renderer.rs:206</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When renderer res in world, then system can call clear via resmut</summary>

<code>crates\engine_render\src\renderer.rs:219</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>shape</strong> (37 tests)</summary>

<blockquote>
<details>
<summary>When circle aabb, then width and height equal double radius</summary>

<code>crates\engine_render\src\shape.rs:346</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no camera entity, then all shapes drawn without culling</summary>

<code>crates\engine_render\src\shape.rs:1047</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When polygon aabb, then matches point extents</summary>

<code>crates\engine_render\src\shape.rs:359</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When polygon shape variant debug formatted, then snapshot matches</summary>

<code>crates\engine_render\src\shape.rs:176</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape at known position, then vertices offset by translation</summary>

<code>crates\engine_render\src\shape.rs:712</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape at negative pos inside view, then drawn</summary>

<code>crates\engine_render\src\shape.rs:894</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape barely inside view due to radius, then drawn</summary>

<code>crates\engine_render\src\shape.rs:828</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape barely inside view due to y radius, then drawn</summary>

<code>crates\engine_render\src\shape.rs:850</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape circle serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\shape.rs:195</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape fully inside camera view, then drawn</summary>

<code>crates\engine_render\src\shape.rs:787</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape fully outside camera view, then not drawn</summary>

<code>crates\engine_render\src\shape.rs:760</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape has additive material, then set blend mode called with additive</summary>

<code>crates\engine_render\src\shape.rs:468</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape has material, then set shader called with material shader</summary>

<code>crates\engine_render\src\shape.rs:920</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape has material uniforms, then set material uniforms called</summary>

<code>crates\engine_render\src\shape.rs:957</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape has no material, then set blend mode called with alpha</summary>

<code>crates\engine_render\src\shape.rs:453</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape has no material, then set shader called with default</summary>

<code>crates\engine_render\src\shape.rs:942</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape has no render layer, then treated as world layer</summary>

<code>crates\engine_render\src\shape.rs:679</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape has texture bindings, then bind material texture called</summary>

<code>crates\engine_render\src\shape.rs:979</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape near view min edge, then drawn</summary>

<code>crates\engine_render\src\shape.rs:872</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape polygon serialized to ron, then deserializes with point order preserved</summary>

<code>crates\engine_render\src\shape.rs:211</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape with effective visibility false, then not drawn</summary>

<code>crates\engine_render\src\shape.rs:564</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape with global transform, then draw shape called once</summary>

<code>crates\engine_render\src\shape.rs:524</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape with known color, then draw shape receives matching color</summary>

<code>crates\engine_render\src\shape.rs:738</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape without global transform, then draw shape not called</summary>

<code>crates\engine_render\src\shape.rs:544</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating circle, then all indices within vertex bounds</summary>

<code>crates\engine_render\src\shape.rs:258</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating circle, then index count is multiple of three</summary>

<code>crates\engine_render\src\shape.rs:246</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating circle, then produces nonempty vertices and indices</summary>

<code>crates\engine_render\src\shape.rs:233</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating larger circle, then more vertices than smaller</summary>

<code>crates\engine_render\src\shape.rs:288</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating polygon with fewer than three points, then returns empty mesh</summary>

<code>crates\engine_render\src\shape.rs:378</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating quad polygon, then valid triangulated mesh</summary>

<code>crates\engine_render\src\shape.rs:321</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating triangle polygon, then produces three vertices and three indices</summary>

<code>crates\engine_render\src\shape.rs:302</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When tessellating zero radius circle, then does not panic</summary>

<code>crates\engine_render\src\shape.rs:276</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two shapes on different layers, then background drawn before world</summary>

<code>crates\engine_render\src\shape.rs:609</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two shapes same layer different sort order, then lower drawn first</summary>

<code>crates\engine_render\src\shape.rs:643</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two shapes with different blend modes, then set blend mode in sorted order</summary>

<code>crates\engine_render\src\shape.rs:490</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two shapes with different shaders, then shader dominates blend in sort</summary>

<code>crates\engine_render\src\shape.rs:1004</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two visible shapes, then draw shape called twice</summary>

<code>crates\engine_render\src\shape.rs:588</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>sprite</strong> (42 tests)</summary>

<blockquote>
<details>
<summary>When different layers, then layer overrides blend mode order</summary>

<code>crates\engine_render\src\sprite.rs:694</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has effective visibility false, then draw sprite not called</summary>

*EffectiveVisibility(false) is the earliest cull — filtered before sorting or frustum tests*

<code>crates\engine_render\src\sprite.rs:187</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has effective visibility true, then draw sprite called</summary>

<code>crates\engine_render\src\sprite.rs:231</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has no effective visibility, then draw sprite called</summary>

<code>crates\engine_render\src\sprite.rs:211</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has sprite and global transform, then draw sprite called once</summary>

<code>crates\engine_render\src\sprite.rs:146</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has sprite but no global transform, then draw sprite not called</summary>

<code>crates\engine_render\src\sprite.rs:166</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When invisible entity with material, then no blend or draw calls</summary>

<code>crates\engine_render\src\sprite.rs:777</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no camera entity, then all sprites drawn without culling</summary>

*Without a Camera2D entity, frustum culling is disabled entirely — all sprites are drawn*

<code>crates\engine_render\src\sprite.rs:867</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When same layer and blend different sort order, then lower sort first</summary>

<code>crates\engine_render\src\sprite.rs:734</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When same shader different blend, then blend sorts within shader group</summary>

<code>crates\engine_render\src\sprite.rs:1156</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite at known position, then rect xy match translation</summary>

<code>crates\engine_render\src\sprite.rs:415</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite fully inside camera view, then draw sprite called</summary>

<code>crates\engine_render\src\sprite.rs:839</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite fully outside camera view, then draw sprite not called</summary>

*Frustum culling skips draw calls for sprites whose AABB falls entirely outside the camera view rect*

<code>crates\engine_render\src\sprite.rs:812</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has additive material, then set blend mode called with additive</summary>

<code>crates\engine_render\src\sprite.rs:561</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has empty uniforms, then set material uniforms not called</summary>

<code>crates\engine_render\src\sprite.rs:1091</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has material, then set shader called with material shader</summary>

<code>crates\engine_render\src\sprite.rs:954</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has material uniforms, then set material uniforms called</summary>

<code>crates\engine_render\src\sprite.rs:1054</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has multiple texture bindings, then all forwarded in order</summary>

<code>crates\engine_render\src\sprite.rs:1213</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has multiply material, then set blend mode called with multiply</summary>

<code>crates\engine_render\src\sprite.rs:583</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has no material, then bind material texture not called</summary>

<code>crates\engine_render\src\sprite.rs:1244</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has no material, then set blend mode called with alpha</summary>

<code>crates\engine_render\src\sprite.rs:546</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has no material, then set material uniforms not called</summary>

<code>crates\engine_render\src\sprite.rs:1076</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has no material, then set shader called with default</summary>

<code>crates\engine_render\src\sprite.rs:976</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has no render layer, then treated as world layer</summary>

<code>crates\engine_render\src\sprite.rs:347</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has no sort order, then treated as zero</summary>

<code>crates\engine_render\src\sprite.rs:380</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite has texture bindings, then bind material texture called</summary>

<code>crates\engine_render\src\sprite.rs:1188</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite just inside view bottom edge due to height, then drawn</summary>

<code>crates\engine_render\src\sprite.rs:922</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite just inside view right edge due to width, then drawn</summary>

<code>crates\engine_render\src\sprite.rs:890</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_render\src\sprite.rs:127</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite straddles camera view edge, then draw sprite called</summary>

*Edge-touching sprites are drawn — conservative culling avoids popping artifacts at view boundaries*

<code>crates\engine_render\src\sprite.rs:1260</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite with known color, then rect color matches</summary>

<code>crates\engine_render\src\sprite.rs:457</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite with known dimensions, then rect size matches</summary>

<code>crates\engine_render\src\sprite.rs:434</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sprite with known uv rect, then draw sprite receives matching uv</summary>

<code>crates\engine_render\src\sprite.rs:479</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites on different layers, then background drawn before world</summary>

*RenderLayer is the primary sort key — Background draws before World regardless of SortOrder*

<code>crates\engine_render\src\sprite.rs:277</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites same layer different sort order, then lower drawn first</summary>

<code>crates\engine_render\src\sprite.rs:311</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites with different blend modes, then set blend mode called in sorted order</summary>

<code>crates\engine_render\src\sprite.rs:605</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites with different shaders, then set shader called for each</summary>

<code>crates\engine_render\src\sprite.rs:991</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites with different shaders, then shader dominates blend in sort</summary>

<code>crates\engine_render\src\sprite.rs:1113</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites with same blend mode, then both drawn</summary>

<code>crates\engine_render\src\sprite.rs:659</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites with same blend mode, then set blend mode called once</summary>

<code>crates\engine_render\src\sprite.rs:628</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two sprites with same shader, then set shader called once</summary>

<code>crates\engine_render\src\sprite.rs:1023</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two visible sprites, then draw sprite called twice</summary>

<code>crates\engine_render\src\sprite.rs:255</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>testing</strong> (17 tests)</summary>

<blockquote>
<details>
<summary>When apply post process called, then log records apply post process</summary>

<code>crates\engine_render\src\testing.rs:379</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bind material texture called with capture, then entry matches</summary>

<code>crates\engine_render\src\testing.rs:435</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When clear called, then log records clear string</summary>

<code>crates\engine_render\src\testing.rs:218</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When clear called with color capture, then color is stored</summary>

<code>crates\engine_render\src\testing.rs:450</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When draw rect called, then log records draw rect string</summary>

<code>crates\engine_render\src\testing.rs:231</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When draw shape called, then log records draw shape string</summary>

<code>crates\engine_render\src\testing.rs:297</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When draw shape called with capture, then color matches</summary>

<code>crates\engine_render\src\testing.rs:314</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When draw sprite called, then log records draw sprite string</summary>

<code>crates\engine_render\src\testing.rs:271</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When present called, then log records present string</summary>

<code>crates\engine_render\src\testing.rs:245</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When resize called, then log records resize string</summary>

<code>crates\engine_render\src\testing.rs:258</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set blend mode called, then log records set blend mode</summary>

<code>crates\engine_render\src\testing.rs:331</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set blend mode called twice with capture, then both calls recorded in order</summary>

<code>crates\engine_render\src\testing.rs:344</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set material uniforms called with capture, then bytes match</summary>

<code>crates\engine_render\src\testing.rs:420</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set shader called, then log records set shader</summary>

<code>crates\engine_render\src\testing.rs:392</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set shader called with capture, then handle matches</summary>

<code>crates\engine_render\src\testing.rs:405</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When set view projection called, then log records set view projection</summary>

<code>crates\engine_render\src\testing.rs:284</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When upload atlas called, then log records upload atlas</summary>

<code>crates\engine_render\src\testing.rs:360</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>visual_regression</strong> (17 tests)</summary>

<blockquote>
<details>
<summary>When clearing with red, then readback pixels are all red</summary>

<code>crates\engine_render\src\visual_regression.rs:690</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When comparing different buffers, then ssim score is less than one</summary>

<code>crates\engine_render\src\visual_regression.rs:601</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When comparing identical buffers, then ssim score is one</summary>

<code>crates\engine_render\src\visual_regression.rs:585</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When comparing largely different buffers, then ssim below threshold</summary>

<code>crates\engine_render\src\visual_regression.rs:760</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When comparing slightly different buffers, then ssim above threshold</summary>

<code>crates\engine_render\src\visual_regression.rs:617</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When computing padded row bytes, then returns multiple of 256</summary>

<code>crates\engine_render\src\visual_regression.rs:634</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When creating headless renderer, then viewport matches</summary>

<code>crates\engine_render\src\visual_regression.rs:680</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When drawing white rect on black, then rect region is white</summary>

<code>crates\engine_render\src\visual_regression.rs:783</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading nonexistent golden, then returns error</summary>

<code>crates\engine_render\src\visual_regression.rs:748</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When loading saved golden, then pixels match original</summary>

<code>crates\engine_render\src\visual_regression.rs:728</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rendered frame compared to golden, then ssim passes threshold</summary>

<code>crates\engine_render\src\visual_regression.rs:843</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rendered frame differs from golden, then ssim fails threshold</summary>

<code>crates\engine_render\src\visual_regression.rs:864</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rendering circle shape, then center pixel is non background</summary>

<code>crates\engine_render\src\visual_regression.rs:887</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rendering same scene twice, then buffers are identical</summary>

<code>crates\engine_render\src\visual_regression.rs:823</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When saving golden image, then file exists at expected path</summary>

<code>crates\engine_render\src\visual_regression.rs:711</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When stripping row padding, then produces packed rgba</summary>

<code>crates\engine_render\src\visual_regression.rs:653</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When width already aligned, then padded row bytes unchanged</summary>

<code>crates\engine_render\src\visual_regression.rs:644</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>wgpu_renderer</strong> (22 tests)</summary>

<blockquote>
<details>
<summary>When all same blend mode, then single batch</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1827</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When any rect, then uv rect is full texture</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1550</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When batch cleared, then vertex and index counts are zero</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1701</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When batch is empty, then is empty returns true</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1717</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When blend mode additive, then blend state uses src alpha one</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1763</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When blend mode alpha, then blend state is alpha blending</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1754</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When blend mode multiply, then blend state uses dst zero</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1784</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When colored rect, then instance color matches</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1568</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When fullscreen quad indices resolved, then two ccw triangles cover quad</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1735</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When fullscreen quad vertices queried, then four corners span ndc</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1622</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When mixed blend modes, then batches split at boundaries</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1805</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When negative dimensions, then stored without clamping</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1604</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When no items, then empty batches</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1842</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When offset rect, then instance encodes position and size</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1532</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When quad indices used, then two triangles cover unit square</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1495</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When rect at origin, then instance encodes world coordinates</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1514</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape shader parsed, then no error</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1726</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape vertex size checked, then exactly 24 bytes</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1639</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When shape vertices cast to bytes, then no panic</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1648</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When single shape pushed, then vertex and index counts match input</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1669</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two shapes pushed, then second indices are offset by first vertex count</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1684</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When zero size rect, then no panic and zero dimensions</summary>

<code>crates\engine_render\src\wgpu_renderer.rs:1586</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_scene</strong> (42 tests)</summary>

<blockquote>
<details>
<summary><strong>hierarchy</strong> (10 tests)</summary>

<blockquote>
<details>
<summary>When arbitrary child of assignments, then children vec is sorted</summary>

<code>crates\engine_scene\src\hierarchy.rs:197</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When child entity is despawned, then parent children no longer contains that child</summary>

<code>crates\engine_scene\src\hierarchy.rs:176</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When child of is removed, then parent children no longer contains that child</summary>

<code>crates\engine_scene\src\hierarchy.rs:140</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has child of, then hierarchy system adds it to parent children</summary>

<code>crates\engine_scene\src\hierarchy.rs:40</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When last child of is removed, then parent children component is removed</summary>

<code>crates\engine_scene\src\hierarchy.rs:160</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When multiple children belong to parent, then children vec is sorted by entity</summary>

<code>crates\engine_scene\src\hierarchy.rs:100</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When only child is despawned, then parent children component is removed</summary>

<code>crates\engine_scene\src\hierarchy.rs:232</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When system runs twice with no changes, then children remains stable</summary>

<code>crates\engine_scene\src\hierarchy.rs:122</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two children share same parent, then children contains both</summary>

<code>crates\engine_scene\src\hierarchy.rs:57</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two parents each have one child, then each parent children is independent</summary>

<code>crates\engine_scene\src\hierarchy.rs:77</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>render_order</strong> (5 tests)</summary>

<blockquote>
<details>
<summary>When entities sorted by render layer and sort order, then order is deterministic</summary>

<code>crates\engine_scene\src\render_order.rs:67</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When render layer variants serialized to ron, then each deserializes to matching variant</summary>

<code>crates\engine_scene\src\render_order.rs:24</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When render layers compared, then background less than world less than characters less than foreground less than ui</summary>

<code>crates\engine_scene\src\render_order.rs:52</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sort order serialized to ron, then deserializes to equal value</summary>

<code>crates\engine_scene\src\render_order.rs:39</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When sort order values compared, then lower i32 value sorts before higher</summary>

<code>crates\engine_scene\src\render_order.rs:61</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>spawn_child</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>When spawn child called, then new entity also contains the provided bundle</summary>

<code>crates\engine_scene\src\spawn_child.rs:41</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When spawn child called, then new entity has child of pointing to parent</summary>

<code>crates\engine_scene\src\spawn_child.rs:22</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When spawn child used, then hierarchy system picks up the new child</summary>

<code>crates\engine_scene\src\spawn_child.rs:55</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>transform_propagation</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>When child has identity transform, then global transform equals parent</summary>

<code>crates\engine_scene\src\transform_propagation.rs:112</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When child has translation and parent has translation, then both accumulate</summary>

<code>crates\engine_scene\src\transform_propagation.rs:134</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When entity has no transform2d, then propagation system does not insert global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:99</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When hierarchy system runs before propagation, then children receive global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:303</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When multiple root entities, then each gets independent global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:276</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When parent has scale and child has translation, then child position is scaled</summary>

<code>crates\engine_scene\src\transform_propagation.rs:164</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When root entity has identity transform, then global transform equals affine2 identity</summary>

<code>crates\engine_scene\src\transform_propagation.rs:66</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When root entity has transform2d, then propagation system inserts global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:53</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When root entity has translation only, then global transform matches</summary>

<code>crates\engine_scene\src\transform_propagation.rs:80</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When three level hierarchy, then grandchild accumulates all ancestors</summary>

<code>crates\engine_scene\src\transform_propagation.rs:193</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When transform updated and system reruns, then global transform reflects new value</summary>

<code>crates\engine_scene\src\transform_propagation.rs:334</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two siblings, then each gets independent global transform</summary>

<code>crates\engine_scene\src\transform_propagation.rs:232</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>visibility</strong> (12 tests)</summary>

<blockquote>
<details>
<summary>When child has no visible component and parent is hidden, then child effective visibility is false</summary>

<code>crates\engine_scene\src\visibility.rs:253</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When child has no visible component and parent is visible, then child effective visibility is true</summary>

<code>crates\engine_scene\src\visibility.rs:236</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When hierarchy system runs before visibility system, then children receive effective visibility</summary>

<code>crates\engine_scene\src\visibility.rs:197</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When parent is hidden and child is visible, then child effective visibility is false</summary>

<code>crates\engine_scene\src\visibility.rs:126</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When parent is visible and child is hidden, then child effective visibility is false</summary>

<code>crates\engine_scene\src\visibility.rs:142</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When parent visibility changed and system reruns, then child effective visibility updates</summary>

<code>crates\engine_scene\src\visibility.rs:214</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When root entity has default visible, then visibility system inserts effective visibility true</summary>

<code>crates\engine_scene\src\visibility.rs:65</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When root entity has no visible component, then visibility system inserts effective visibility true</summary>

<code>crates\engine_scene\src\visibility.rs:95</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When root entity has visible false, then visibility system inserts effective visibility false</summary>

<code>crates\engine_scene\src\visibility.rs:80</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When three level hierarchy with hidden root, then grandchild effective visibility is false</summary>

<code>crates\engine_scene\src\visibility.rs:158</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two siblings one hidden, then each gets independent effective visibility</summary>

<code>crates\engine_scene\src\visibility.rs:176</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When visible parent has visible child, then child effective visibility is true</summary>

<code>crates\engine_scene\src\visibility.rs:110</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>engine_ui</strong> (69 tests)</summary>

<blockquote>
<details>
<summary><strong>anchor</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>When all nine anchors, then all offsets distinct</summary>

<code>crates\engine_ui\src\anchor.rs:115</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bottom center anchor, then half width full height</summary>

<code>crates\engine_ui\src\anchor.rs:74</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When bottom right anchor and any size, then offset is negative size</summary>

<code>crates\engine_ui\src\anchor.rs:99</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When center anchor, then negative half size</summary>

<code>crates\engine_ui\src\anchor.rs:38</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When top left anchor and any size, then offset is zero</summary>

<code>crates\engine_ui\src\anchor.rs:87</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When top left anchor, then zero offset</summary>

<code>crates\engine_ui\src\anchor.rs:50</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When top right anchor, then negative width</summary>

<code>crates\engine_ui\src\anchor.rs:62</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>button</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>When button disabled, then disabled color used regardless of interaction</summary>

<code>crates\engine_ui\src\button.rs:164</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button hovered, then hovered color used</summary>

<code>crates\engine_ui\src\button.rs:118</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button invisible, then no draw</summary>

<code>crates\engine_ui\src\button.rs:187</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button not hovered, then normal color used</summary>

<code>crates\engine_ui\src\button.rs:94</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button pressed, then pressed color used</summary>

<code>crates\engine_ui\src\button.rs:141</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button rendered, then position and size match node</summary>

<code>crates\engine_ui\src\button.rs:209</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When button roundtrip ron, then disabled preserved</summary>

<code>crates\engine_ui\src\button.rs:81</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>flex_layout</strong> (7 tests)</summary>

<blockquote>
<details>
<summary>When any row children, then output length matches and x offsets increase</summary>

<code>crates\engine_ui\src\flex_layout.rs:161</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When column with gap, then children vertical</summary>

<code>crates\engine_ui\src\flex_layout.rs:94</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When empty children, then empty offsets</summary>

<code>crates\engine_ui\src\flex_layout.rs:196</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When row no gap, then children horizontal</summary>

<code>crates\engine_ui\src\flex_layout.rs:56</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When row with gap, then gap between children</summary>

<code>crates\engine_ui\src\flex_layout.rs:75</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When row with margins, then margins in spacing</summary>

<code>crates\engine_ui\src\flex_layout.rs:113</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When single child, then offset at origin</summary>

<code>crates\engine_ui\src\flex_layout.rs:144</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>interaction</strong> (16 tests)</summary>

<blockquote>
<details>
<summary>When cursor enters node, then hover enter event emitted</summary>

<code>crates\engine_ui\src\interaction.rs:406</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor inside and left held, then interaction becomes pressed</summary>

<code>crates\engine_ui\src\interaction.rs:201</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor inside node, then interaction becomes hovered</summary>

<code>crates\engine_ui\src\interaction.rs:126</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor leaves node, then hover exit event emitted</summary>

<code>crates\engine_ui\src\interaction.rs:431</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor leaves node, then interaction reverts to none</summary>

<code>crates\engine_ui\src\interaction.rs:343</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor on node boundary, then interaction becomes hovered</summary>

<code>crates\engine_ui\src\interaction.rs:151</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor outside and left held, then interaction remains none</summary>

<code>crates\engine_ui\src\interaction.rs:227</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When cursor outside node, then interaction remains none</summary>

<code>crates\engine_ui\src\interaction.rs:176</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When different node clicked, then focus transfers</summary>

<code>crates\engine_ui\src\interaction.rs:490</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When disabled button, then interaction stays none</summary>

<code>crates\engine_ui\src\interaction.rs:559</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When effective visibility false, then not hit tested</summary>

<code>crates\engine_ui\src\interaction.rs:278</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When interaction roundtrip ron, then variant preserved</summary>

<code>crates\engine_ui\src\interaction.rs:540</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When just pressed inside, then clicked event emitted</summary>

<code>crates\engine_ui\src\interaction.rs:377</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When node clicked, then focus state updated</summary>

<code>crates\engine_ui\src\interaction.rs:462</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When node has center anchor, then hit test accounts for offset</summary>

<code>crates\engine_ui\src\interaction.rs:253</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two overlapping nodes, then both receive interaction</summary>

<code>crates\engine_ui\src\interaction.rs:304</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>layout</strong> (6 tests)</summary>

<blockquote>
<details>
<summary>When child has margin, then margin in spacing</summary>

<code>crates\engine_ui\src\layout.rs:206</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When column layout, then vertical stacking</summary>

<code>crates\engine_ui\src\layout.rs:161</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When parent offset, then children relative</summary>

<code>crates\engine_ui\src\layout.rs:184</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When row layout, then first child at origin</summary>

<code>crates\engine_ui\src\layout.rs:92</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When row layout, then second child offset by first width</summary>

<code>crates\engine_ui\src\layout.rs:115</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When row layout with gap, then gap included</summary>

<code>crates\engine_ui\src\layout.rs:138</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>margin</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When asymmetric margin, then correct pairs</summary>

<code>crates\engine_ui\src\margin.rs:37</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When zero margin, then totals zero</summary>

<code>crates\engine_ui\src\margin.rs:27</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>panel</strong> (4 tests)</summary>

<blockquote>
<details>
<summary>When panel invisible, then no draw</summary>

<code>crates\engine_ui\src\panel.rs:172</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When panel no border, then only background drawn</summary>

<code>crates\engine_ui\src\panel.rs:119</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When panel roundtrip ron, then border preserved</summary>

<code>crates\engine_ui\src\panel.rs:103</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When panel with border, then background plus border rects drawn</summary>

<code>crates\engine_ui\src\panel.rs:145</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>progress_bar</strong> (6 tests)</summary>

<blockquote>
<details>
<summary>When progress bar at full, then filled rect matches node width</summary>

<code>crates\engine_ui\src\progress_bar.rs:166</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When progress bar at half, then filled rect is half width</summary>

<code>crates\engine_ui\src\progress_bar.rs:139</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When progress bar at zero, then only background drawn</summary>

<code>crates\engine_ui\src\progress_bar.rs:113</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When progress bar exceeds max, then filled rect capped at node width</summary>

<code>crates\engine_ui\src\progress_bar.rs:193</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When progress bar invisible, then no draw</summary>

<code>crates\engine_ui\src\progress_bar.rs:220</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When progress bar roundtrip ron, then value and max preserved</summary>

<code>crates\engine_ui\src\progress_bar.rs:97</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>render</strong> (8 tests)</summary>

<blockquote>
<details>
<summary>When center anchor, then rect adjusted by half size</summary>

<code>crates\engine_ui\src\render.rs:125</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When effective visibility false, then no draw</summary>

<code>crates\engine_ui\src\render.rs:195</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When top left anchor, then rect at transform position</summary>

<code>crates\engine_ui\src\render.rs:101</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When two ui nodes, then both drawn</summary>

<code>crates\engine_ui\src\render.rs:217</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When ui node has background, then draw rect called</summary>

<code>crates\engine_ui\src\render.rs:60</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When ui node no background, then no draw</summary>

<code>crates\engine_ui\src\render.rs:81</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When ui node rendered, then rect color matches background</summary>

<code>crates\engine_ui\src\render.rs:172</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When ui node rendered, then rect size matches node</summary>

<code>crates\engine_ui\src\render.rs:149</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>theme</strong> (1 tests)</summary>

<blockquote>
<details>
<summary>When ui theme roundtrip ron, then all fields preserved</summary>

<code>crates\engine_ui\src\theme.rs:34</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>ui_event</strong> (2 tests)</summary>

<blockquote>
<details>
<summary>When clicked event pushed, then drain yields exact event and buffer is empty</summary>

<code>crates\engine_ui\src\ui_event.rs:37</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When drained twice, then second drain is empty</summary>

<code>crates\engine_ui\src\ui_event.rs:53</code>

</details>
</blockquote>

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>ui_node</strong> (3 tests)</summary>

<blockquote>
<details>
<summary>When flex layout roundtrip ron, then preserved</summary>

<code>crates\engine_ui\src\ui_node.rs:59</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When text roundtrip ron, then preserved</summary>

<code>crates\engine_ui\src\ui_node.rs:75</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When ui node roundtrip ron, then preserved</summary>

<code>crates\engine_ui\src\ui_node.rs:36</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

<details>
<summary><strong>living_docs</strong> (32 tests)</summary>

<blockquote>
<details>
<summary><strong>doc_tests</strong> (1 tests)</summary>

- Handle::Handle (line 6)

</details>
</blockquote>

<blockquote>
<details>
<summary><strong>tests</strong> (31 tests)</summary>

<blockquote>
<details>
<summary>When annotation present, then markdown includes it as subtext</summary>

<code>tools\living-docs\src\lib.rs:816</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When binary unittest line parsed, then returns crate name</summary>

<code>tools\living-docs\src\lib.rs:339</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When crate doc generated, then produces heading and subheadings</summary>

<code>tools\living-docs\src\lib.rs:552</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When crate has no tests, then crate still appears</summary>

<code>tools\living-docs\src\lib.rs:517</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When doc test line parsed, then returns doc tests module</summary>

<code>tools\living-docs\src\lib.rs:403</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When empty line parsed, then returns none</summary>

<code>tools\living-docs\src\lib.rs:435</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When full pipeline, then produces valid markdown</summary>

<code>tools\living-docs\src\lib.rs:689</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When lib unittest line parsed, then returns crate name</summary>

<code>tools\living-docs\src\lib.rs:326</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When markdown generated, then crate sections are collapsible</summary>

<code>tools\living-docs\src\lib.rs:761</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When markdown generated, then header includes count and date</summary>

<code>tools\living-docs\src\lib.rs:612</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When markdown generated, then module sections are collapsible</summary>

<code>tools\living-docs\src\lib.rs:790</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When markdown generated, then tests appear as list items</summary>

<code>tools\living-docs\src\lib.rs:587</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When module section, then wrapped in blockquote</summary>

<code>tools\living-docs\src\lib.rs:950</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When modules in crate, then alphabetical order</summary>

<code>tools\living-docs\src\lib.rs:663</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When multi crate output parsed, then groups tests by crate</summary>

<code>tools\living-docs\src\lib.rs:493</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When multiple crates, then alphabetical order</summary>

<code>tools\living-docs\src\lib.rs:627</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When name without when prefix converted, then capitalizes first letter</summary>

<code>tools\living-docs\src\lib.rs:464</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When nested module test line parsed, then uses top level module</summary>

<code>tools\living-docs\src\lib.rs:384</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When non running line parsed, then returns none</summary>

<code>tools\living-docs\src\lib.rs:352</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When source has doc annotation, then parse annotations extracts it</summary>

<code>tools\living-docs\src\lib.rs:721</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When source has no annotations, then map is empty</summary>

<code>tools\living-docs\src\lib.rs:743</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When source has test fn, then parse test locations returns location</summary>

<code>tools\living-docs\src\lib.rs:842</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When standard test line parsed, then extracts module and name</summary>

<code>tools\living-docs\src\lib.rs:365</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When summary line parsed, then returns none</summary>

<code>tools\living-docs\src\lib.rs:422</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When test foldout, then wrapped in blockquote</summary>

<code>tools\living-docs\src\lib.rs:975</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When test has annotation and source, then both in foldout</summary>

<code>tools\living-docs\src\lib.rs:918</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When test has no annotation or source, then rendered as plain list item</summary>

<code>tools\living-docs\src\lib.rs:892</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When test has source, then markdown shows location in foldout</summary>

<code>tools\living-docs\src\lib.rs:861</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When warnings interspersed, then they are ignored</summary>

<code>tools\living-docs\src\lib.rs:533</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When when, then name converted then capitalizes when and inserts comma</summary>

<code>tools\living-docs\src\lib.rs:448</code>

</details>
</blockquote>
<blockquote>
<details>
<summary>When when, then with long middle then comma before then</summary>

<code>tools\living-docs\src\lib.rs:477</code>

</details>
</blockquote>

</details>
</blockquote>

</details>

