<script lang="ts" module>
  // Bootstrap Icons (MIT, https://icons.getbootstrap.com). Импортируем только нужные — в сборку
  // попадают лишь эти SVG, а не весь набор из 2000+ иконок.
  import i_play from 'bootstrap-icons/icons/play-fill.svg?raw';
  import i_pause from 'bootstrap-icons/icons/pause-fill.svg?raw';
  import i_next from 'bootstrap-icons/icons/skip-end-fill.svg?raw';
  import i_prev from 'bootstrap-icons/icons/skip-start-fill.svg?raw';
  import i_search from 'bootstrap-icons/icons/search.svg?raw';
  import i_home from 'bootstrap-icons/icons/house-door.svg?raw';
  import i_homeFill from 'bootstrap-icons/icons/house-door-fill.svg?raw';
  import i_disc from 'bootstrap-icons/icons/disc.svg?raw';
  import i_nowPlaying from 'bootstrap-icons/icons/play-circle.svg?raw';
  import i_discFill from 'bootstrap-icons/icons/disc-fill.svg?raw';
  import i_users from 'bootstrap-icons/icons/people.svg?raw';
  import i_usersFill from 'bootstrap-icons/icons/people-fill.svg?raw';
  import i_chat from 'bootstrap-icons/icons/chat.svg?raw';
  import i_queue from 'bootstrap-icons/icons/music-note-list.svg?raw';
  import i_plus from 'bootstrap-icons/icons/plus-lg.svg?raw';
  import i_x from 'bootstrap-icons/icons/x-lg.svg?raw';
  import i_minimize from 'bootstrap-icons/icons/dash-lg.svg?raw';
  import i_maximize from 'bootstrap-icons/icons/square.svg?raw';
  import i_shuffle from 'bootstrap-icons/icons/shuffle.svg?raw';
  import i_repeat from 'bootstrap-icons/icons/repeat.svg?raw';
  import i_repeat1 from 'bootstrap-icons/icons/repeat-1.svg?raw';
  import i_volume from 'bootstrap-icons/icons/volume-up.svg?raw';
  import i_volumeLow from 'bootstrap-icons/icons/volume-down.svg?raw';
  import i_mute from 'bootstrap-icons/icons/volume-mute.svg?raw';
  import i_settings from 'bootstrap-icons/icons/gear.svg?raw';
  import i_link from 'bootstrap-icons/icons/link-45deg.svg?raw';
  import i_copy from 'bootstrap-icons/icons/copy.svg?raw';
  import i_logout from 'bootstrap-icons/icons/box-arrow-right.svg?raw';
  import i_crown from 'bootstrap-icons/icons/star-fill.svg?raw';
  import i_trash from 'bootstrap-icons/icons/trash3.svg?raw';
  import i_playNext from 'bootstrap-icons/icons/arrow-return-right.svg?raw';
  import i_check from 'bootstrap-icons/icons/check-lg.svg?raw';
  import i_smile from 'bootstrap-icons/icons/emoji-smile.svg?raw';
  import i_arrowRight from 'bootstrap-icons/icons/arrow-right.svg?raw';
  import i_send from 'bootstrap-icons/icons/send-fill.svg?raw';
  import i_sparkles from 'bootstrap-icons/icons/stars.svg?raw';
  import i_grip from 'bootstrap-icons/icons/grip-vertical.svg?raw';
  import i_wave from 'bootstrap-icons/icons/soundwave.svg?raw';
  import i_download from 'bootstrap-icons/icons/download.svg?raw';
  import i_refresh from 'bootstrap-icons/icons/arrow-clockwise.svg?raw';
  import i_youtube from 'bootstrap-icons/icons/youtube.svg?raw';
  import i_cloud from 'bootstrap-icons/icons/cloud.svg?raw';
  import i_music from 'bootstrap-icons/icons/music-note-beamed.svg?raw';
  import i_lyrics from 'bootstrap-icons/icons/quote.svg?raw';
  import i_upload from 'bootstrap-icons/icons/upload.svg?raw';
  import i_dash from 'bootstrap-icons/icons/dash.svg?raw';
  import i_plusSm from 'bootstrap-icons/icons/plus.svg?raw';
  import i_note from 'bootstrap-icons/icons/music-note.svg?raw';
  import i_file from 'bootstrap-icons/icons/file-earmark-music.svg?raw';

  function inner(svg: string) {
    const s = svg.indexOf('>') + 1;
    return svg.slice(s, svg.lastIndexOf('</svg>')).trim();
  }

  const RAW: Record<string, string> = {
    play: i_play,
    pause: i_pause,
    next: i_next,
    prev: i_prev,
    search: i_search,
    home: i_home,
    homeFill: i_homeFill,
    disc: i_disc,
    nowPlaying: i_nowPlaying,
    discFill: i_discFill,
    users: i_users,
    usersFill: i_usersFill,
    chat: i_chat,
    queue: i_queue,
    plus: i_plus,
    x: i_x,
    minimize: i_minimize,
    maximize: i_maximize,
    shuffle: i_shuffle,
    repeat: i_repeat,
    repeat1: i_repeat1,
    volume: i_volume,
    volumeLow: i_volumeLow,
    mute: i_mute,
    settings: i_settings,
    link: i_link,
    copy: i_copy,
    logout: i_logout,
    crown: i_crown,
    trash: i_trash,
    playNext: i_playNext,
    check: i_check,
    smile: i_smile,
    arrowRight: i_arrowRight,
    send: i_send,
    sparkles: i_sparkles,
    grip: i_grip,
    wave: i_wave,
    download: i_download,
    refresh: i_refresh,
    youtube: i_youtube,
    cloud: i_cloud,
    music: i_music,
    lyrics: i_lyrics,
    upload: i_upload,
    dash: i_dash,
    plusSm: i_plusSm,
    note: i_note,
    file: i_file,
  };
  const PATHS: Record<string, string> = Object.fromEntries(Object.entries(RAW).map(([k, v]) => [k, inner(v)]));
</script>

<script lang="ts">
  // stroke оставлен для совместимости вызовов — Bootstrap Icons залитые, толщина не меняется.
  let { name, size = 18 }: { name: string; size?: number; stroke?: number } = $props();
</script>

<svg class="bi" width={size} height={size} viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
  {@html PATHS[name] ?? ''}
</svg>

<style>
  .bi {
    flex: none;
    display: block;
  }
</style>
