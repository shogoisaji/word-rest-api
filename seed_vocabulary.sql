-- Vocabulary seed data
-- This script inserts 5 sample vocabulary entries into the database

INSERT INTO vocabulary (en_word, ja_word, en_example, ja_example, created_at, updated_at)
VALUES 
  ('apple', 'りんご', 'I eat an apple every day.', '私は毎日りんごを食べます。', NOW(), NOW()),
  ('book', '本', 'This is an interesting book.', 'これは面白い本です。', NOW(), NOW()),
  ('computer', 'コンピューター', 'I use my computer for work.', '私は仕事でコンピューターを使います。', NOW(), NOW()),
  ('study', '勉強する', 'I study English every morning.', '私は毎朝英語を勉強します。', NOW(), NOW()),
  ('friend', '友達', 'She is my best friend.', '彼女は私の親友です。', NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Verify the data was inserted
SELECT COUNT(*) as total_vocabulary FROM vocabulary;
SELECT * FROM vocabulary ORDER BY created_at DESC LIMIT 5;
