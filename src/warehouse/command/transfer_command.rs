use warehouse::command::Command;
use warehouse::object::{ Object, Bot };
use warehouse::Storage;
use std::sync::{ Arc, Mutex };
use std::mem::swap;

pub struct TransferCommand {
    from: Arc<Mutex<Object>>,
    to: Arc<Mutex<Object>>
}

pub struct BotTransferToCommand {
}


impl BotTransferToCommand {
    pub fn new(from: Arc<Mutex<Bot>>, to: Arc<Mutex<Object>>) -> Box<TransferCommand> {
        Box::new(TransferCommand {
            from: from, to: to
        })
    }
}

pub struct BotTransferFromCommand {
}

impl BotTransferFromCommand {
    pub fn new(from: Arc<Mutex<Object>>, to: Arc<Mutex<Bot>>) -> Box<TransferCommand> {
        Box::new(TransferCommand {
            from: from, to: to
        })
    }
}

impl Command for TransferCommand {
    fn initialize(&mut self) -> Result<(), &'static str> {
        let mut from = self.from.lock().unwrap();
        let mut to = self.to.lock().unwrap();
        match from.lock() {
            Ok(_) => {},
            Err(err) => return Err(err)
        };
        to.lock()
    }
    fn consume(&mut self) -> Result<bool, &'static str> {
        let mut from = self.from.lock().unwrap();
        let mut to = self.to.lock().unwrap();
        {
            let from_storage = from.get_storage();
            let to_storage = to.get_storage();
            swap(&mut from_storage.items, &mut to_storage.items);
        }
        from.unlock().unwrap();
        to.unlock().unwrap();
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use warehouse::object::TestObject;

    #[test]
    fn test_lock() {
        let bot = Bot::new();
        let obj = TestObject::new();
        let mut cmd = BotTransferToCommand::new(bot.clone(), obj.clone());
        cmd.initialize().unwrap();
        let bot_locked = *bot.lock().unwrap().get_lock();
        let obj_locked = *obj.lock().unwrap().get_lock();
        assert!(bot_locked);
        assert!(obj_locked);
    }
    #[test]
    fn test_unlock() {
        let bot = Bot::new();
        let obj = TestObject::new();
        let mut cmd = BotTransferToCommand::new(bot.clone(), obj.clone());
        cmd.initialize().unwrap();
        assert_eq!(cmd.consume().unwrap(), false);
        let bot_locked = *bot.lock().unwrap().get_lock();
        let obj_locked = *obj.lock().unwrap().get_lock();
        assert!(!bot_locked);
        assert!(!obj_locked);
    }
    #[test]
    #[should_panic]
    fn test_race() {
        let bot = Bot::new();
        let obj = TestObject::new();
        let mut cmd = BotTransferToCommand::new(bot.clone(), obj.clone());
        cmd.initialize().unwrap();
        let mut cmd2 = BotTransferToCommand::new(bot.clone(), obj.clone());
        cmd2.initialize().unwrap();
    }
    #[test]
    fn test_storage() {
        let bot = Bot::new();
        let obj = TestObject::new();
        {
            let (mut bot, mut obj) = (bot.lock().unwrap(), obj.lock().unwrap());
            let (bot_storage, obj_storage) = (bot.get_storage(), obj.get_storage());
            bot_storage.items.push(1); bot_storage.items.push(3);
            obj_storage.items.push(2);
        }
        let mut cmd = BotTransferToCommand::new(bot.clone(), obj.clone());
        cmd.initialize().unwrap();
        cmd.consume().unwrap();
        {
            let (bot, obj) = (bot.lock().unwrap(), obj.lock().unwrap());
            let (bot_storage, obj_storage) = (bot.storage(), obj.storage());
            assert_eq!(bot_storage.items, vec![2]);
            assert_eq!(obj_storage.items, vec![1, 3]);
        }
    }
}