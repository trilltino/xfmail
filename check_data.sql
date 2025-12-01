SELECT 'users' as table_name, COUNT(*) as row_count FROM users
UNION ALL
SELECT 'conversations', COUNT(*) FROM conversations
UNION ALL
SELECT 'chat_messages', COUNT(*) FROM chat_messages
UNION ALL
SELECT 'messages', COUNT(*) FROM messages
UNION ALL
SELECT 'friend_requests', COUNT(*) FROM friend_requests;