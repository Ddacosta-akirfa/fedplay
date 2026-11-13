pub const SCHEMA_V1: &str = r#"
CREATE TABLE watched_directories (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL UNIQUE,
    last_scan   INTEGER,
    created_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE artists (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE COLLATE NOCASE,   -- "AC/DC" == "ac/dc"
    name_sort   TEXT NOT NULL,
    bio         TEXT,
    image_path  TEXT,
    mbid        TEXT,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE albums (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    title         TEXT NOT NULL COLLATE NOCASE,
    artist_id     INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    year          INTEGER,
    cover_path    TEXT,
    mbid          TEXT,
    total_tracks  INTEGER CHECK (total_tracks IS NULL OR total_tracks >= 0),
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    UNIQUE (title, artist_id)
) STRICT;

CREATE TABLE tracks (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    path           TEXT NOT NULL UNIQUE,
    directory_id   INTEGER REFERENCES watched_directories(id) ON DELETE CASCADE,
    artist_id      INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,  -- artista da faixa
    album_id       INTEGER NOT NULL REFERENCES albums(id)  ON DELETE CASCADE,  -- o artista do álbum está em albums.artist_id
    title          TEXT,
    track_number   INTEGER CHECK (track_number IS NULL OR track_number >= 0),
    disc_number    INTEGER NOT NULL DEFAULT 1 CHECK (disc_number >= 1),
    genre          TEXT,
    duration_ms    INTEGER CHECK (duration_ms IS NULL OR duration_ms >= 0),
    cover_path     TEXT,            -- só se a faixa tiver capa própria (senão usa a do álbum)
    file_size      INTEGER,
    file_mtime     INTEGER,         -- para rescan incremental (NULL = reverificar)
    codec          TEXT,
    sample_rate    INTEGER,
    bit_depth      INTEGER,
    bitrate        INTEGER,
    missing_since  INTEGER,         -- NULL = presente; senão, desde quando o ficheiro desapareceu
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL
) STRICT;

CREATE TABLE playlists (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    is_smart    INTEGER NOT NULL DEFAULT 0 CHECK (is_smart IN (0, 1)),
    system_key  TEXT UNIQUE CHECK (system_key IN ('favorites', 'recent', 'most_played')),
    cover_path  TEXT,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL,
    CHECK (system_key IS NULL OR is_smart = 1)
) STRICT;

CREATE TABLE playlist_tracks (
    playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    INTEGER NOT NULL REFERENCES tracks(id)    ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_id)
) STRICT, WITHOUT ROWID;

CREATE TABLE favorites (
    track_id    INTEGER PRIMARY KEY REFERENCES tracks(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- Cada linha = uma reprodução. "Mais tocadas" = COUNT(*) GROUP BY track_id.
CREATE TABLE playback_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    played_at   INTEGER NOT NULL
) STRICT;

-- Índices (as FKs têm todas índice, senão os CASCADE ficam lentos)
CREATE INDEX idx_artists_name_sort        ON artists(name_sort);
CREATE INDEX idx_albums_artist            ON albums(artist_id);
CREATE INDEX idx_albums_year              ON albums(year);
CREATE INDEX idx_tracks_artist            ON tracks(artist_id);
CREATE INDEX idx_tracks_album_order       ON tracks(album_id, disc_number, track_number);
CREATE INDEX idx_tracks_directory         ON tracks(directory_id);
CREATE INDEX idx_tracks_missing           ON tracks(missing_since) WHERE missing_since IS NOT NULL;
CREATE INDEX idx_playlist_tracks_position ON playlist_tracks(playlist_id, position);
CREATE INDEX idx_playlist_tracks_track    ON playlist_tracks(track_id);
CREATE INDEX idx_history_track            ON playback_history(track_id, played_at);
CREATE INDEX idx_history_played_at        ON playback_history(played_at DESC);

-- Leitura: mantém os nomes antigos (artist_name, album_title, album_artist, year)
-- e só mostra faixas presentes. ATENÇÃO: se adicionares colunas a `tracks`
-- numa migração futura, recria a view (DROP VIEW + CREATE VIEW), porque o t.* é fixado na criação.
CREATE VIEW v_tracks AS
SELECT t.*,
       ar.name                               AS artist_name,
       al.title                              AS album_title,
       alar.name                             AS album_artist,
       al.year                               AS year,
       COALESCE(t.cover_path, al.cover_path) AS effective_cover
FROM tracks t
JOIN artists ar   ON ar.id   = t.artist_id
JOIN albums  al   ON al.id   = t.album_id
JOIN artists alar ON alar.id = al.artist_id
WHERE t.missing_since IS NULL;
"#;

pub const SEED_V1: &str = r#"
INSERT OR IGNORE INTO playlists (name, description, is_smart, system_key, created_at, updated_at)
VALUES
  ('Favoritas',            'As tuas faixas marcadas como favoritas',    1, 'favorites',   CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER)),
  ('Tocadas Recentemente', 'Histórico das últimas faixas reproduzidas', 1, 'recent',      CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER)),
  ('Mais Tocadas',         'Top 50 faixas mais reproduzidas',           1, 'most_played', CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER));
"#;

/// Correr depois de cada scan: purga ausentes há +30 dias e remove álbuns/artistas órfãos.
pub const CLEANUP_SQL: &str = r#"
DELETE FROM tracks WHERE missing_since IS NOT NULL
  AND missing_since < CAST(strftime('%s','now') AS INTEGER) - 30*86400;
DELETE FROM albums  WHERE id NOT IN (SELECT DISTINCT album_id FROM tracks);
DELETE FROM artists WHERE id NOT IN (SELECT artist_id FROM tracks UNION SELECT artist_id FROM albums);
"#;

// ---------- Upgrade da base antiga (sem user_version) ----------

pub const LEGACY_PREP_SQL: &str = r#"
DROP INDEX IF EXISTS idx_artists_name;
DROP INDEX IF EXISTS idx_artists_name_sort;
DROP INDEX IF EXISTS idx_albums_title;
DROP INDEX IF EXISTS idx_albums_artist;
DROP INDEX IF EXISTS idx_albums_year;
DROP INDEX IF EXISTS idx_tracks_artist_id;
DROP INDEX IF EXISTS idx_tracks_album_id;
DROP INDEX IF EXISTS idx_tracks_artist_name;
DROP INDEX IF EXISTS idx_tracks_album_title;
DROP INDEX IF EXISTS idx_tracks_title;
DROP INDEX IF EXISTS idx_tracks_genre;
DROP INDEX IF EXISTS idx_tracks_year;
DROP INDEX IF EXISTS idx_tracks_codec;
DROP INDEX IF EXISTS idx_playlist_tracks_position;
DROP INDEX IF EXISTS idx_history_track;
DROP INDEX IF EXISTS idx_history_played_at;
ALTER TABLE artists             RENAME TO artists_old;
ALTER TABLE albums              RENAME TO albums_old;
ALTER TABLE tracks              RENAME TO tracks_old;
ALTER TABLE playlists           RENAME TO playlists_old;
ALTER TABLE playlist_tracks     RENAME TO playlist_tracks_old;
ALTER TABLE favorites           RENAME TO favorites_old;
ALTER TABLE playback_history    RENAME TO playback_history_old;
ALTER TABLE watched_directories RENAME TO watched_directories_old;
"#;

pub const LEGACY_COPY_SQL: &str = r#"
INSERT INTO watched_directories (id, path, last_scan, created_at)
SELECT id, path, last_scan, added_at FROM watched_directories_old;

-- Funde artistas que só diferem em maiúsculas (fica o de menor id)
CREATE TEMP TABLE artist_map AS
SELECT a.id AS old_id,
       (SELECT MIN(b.id) FROM artists_old b WHERE b.name = a.name COLLATE NOCASE) AS new_id
FROM artists_old a;

INSERT INTO artists (id, name, name_sort, bio, image_path, mbid, created_at, updated_at)
SELECT o.id, o.name, o.name_sort, o.bio, o.image_path, o.mbid, o.date_added, o.date_modified
FROM artists_old o JOIN artist_map m ON m.old_id = o.id
WHERE o.id = m.new_id;

-- Mesmo para álbuns (depois de fundir os artistas)
CREATE TEMP TABLE album_map AS
SELECT al.id AS old_id,
       am.new_id AS new_artist_id,
       (SELECT MIN(b.id) FROM albums_old b
          JOIN artist_map bm ON bm.old_id = b.artist_id
         WHERE b.title = al.title COLLATE NOCASE AND bm.new_id = am.new_id) AS new_id
FROM albums_old al JOIN artist_map am ON am.old_id = al.artist_id;

INSERT INTO albums (id, title, artist_id, year, cover_path, mbid, total_tracks, created_at, updated_at)
SELECT o.id, o.title, m.new_artist_id, o.year, o.cover_path, o.mbid, o.total_tracks, o.date_added, o.date_modified
FROM albums_old o JOIN album_map m ON m.old_id = o.id
WHERE o.id = m.new_id;

-- Os ids das faixas mantêm-se, por isso favoritos/playlists/histórico continuam válidos.
-- directory_id: a pasta monitorizada mais específica que contém o ficheiro.
INSERT INTO tracks (id, path, directory_id, artist_id, album_id, title, track_number, disc_number,
                    genre, duration_ms, cover_path, file_size, codec, sample_rate, bit_depth, bitrate,
                    created_at, updated_at)
SELECT o.id, o.path,
       (SELECT d.id FROM watched_directories d
         WHERE substr(o.path, 1, length(d.path) + 1) = d.path || '/'
         ORDER BY length(d.path) DESC LIMIT 1),
       am.new_id, alm.new_id, o.title, o.track_number, COALESCE(o.disc_number, 1),
       o.genre, o.duration_ms, o.cover_path, o.file_size, o.codec, o.sample_rate, o.bit_depth, o.bitrate,
       o.date_added, o.date_modified
FROM tracks_old o
JOIN artist_map am ON am.old_id = o.artist_id
JOIN album_map alm ON alm.old_id = o.album_id;

INSERT INTO playlists (id, name, description, is_smart, system_key, cover_path, created_at, updated_at)
SELECT id, name, description, is_smart,
       CASE WHEN is_smart = 1 AND name = 'Favoritas'            THEN 'favorites'
            WHEN is_smart = 1 AND name = 'Tocadas Recentemente' THEN 'recent'
            WHEN is_smart = 1 AND name = 'Mais Tocadas'         THEN 'most_played' END,
       cover_path, created_at, updated_at
FROM playlists_old;

INSERT INTO playlist_tracks (playlist_id, track_id, position, created_at)
SELECT playlist_id, track_id, position, added_at FROM playlist_tracks_old;

INSERT INTO favorites (track_id, created_at)
SELECT track_id, added_at FROM favorites_old;

-- play_count > 1 passa a ser várias linhas, para não perder o "mais tocadas"
WITH RECURSIVE n(i) AS (
    SELECT 1 UNION ALL SELECT i + 1 FROM n
     WHERE i < (SELECT COALESCE(MAX(play_count), 1) FROM playback_history_old)
)
INSERT INTO playback_history (track_id, played_at)
SELECT h.track_id, h.played_at FROM playback_history_old h JOIN n ON n.i <= h.play_count;

DROP TABLE artist_map;
DROP TABLE album_map;
DROP TABLE playback_history_old;
DROP TABLE favorites_old;
DROP TABLE playlist_tracks_old;
DROP TABLE playlists_old;
DROP TABLE tracks_old;
DROP TABLE albums_old;
DROP TABLE artists_old;
DROP TABLE watched_directories_old;
"#;
