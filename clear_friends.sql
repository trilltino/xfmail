-- Clear all friend requests and contacts (for testing)
DELETE FROM friend_requests;
DELETE FROM contacts;
DELETE FROM conversations;
DELETE FROM conversation_participants;
DELETE FROM messages;

SELECT 'All friend requests, contacts, and conversations cleared!' as status;
