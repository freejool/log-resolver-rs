use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ParseError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

// 2. 实现 Display trait，用于用户友好的错误信息展示
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "自定义错误: {}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, "\n  原因: {}", source)?;
        }
        Ok(())
    }
}

// 3. 实现 Error trait，这是将你的类型标记为错误的核心
impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // 如果你的错误是由另一个错误引起的，这里可以返回那个错误的引用
        // 注意这里的类型转换
        self.source
            .as_ref()
            .map(|b| b.as_ref() as &(dyn Error + 'static))
    }

    // description() 方法已废弃，通常不需要实现
    // fn description(&self) -> &str {
    //     &self.message
    // }

    // cause() 方法也已废弃，被 source() 取代
    // fn cause(&self) -> Option<&dyn Error> {
    //     self.source.as_ref().map(|b| b.as_ref())
    // }
}

// 4. (可选) 为你的错误类型提供一些构造函数，使其更易于创建
impl ParseError {
    // 创建一个简单的错误
    pub fn new(message: &str) -> Self {
        ParseError {
            message: message.to_string(),
            source: None,
        }
    }

    // 创建一个包装了其他错误的错误
    pub fn with_source<E>(message: &str, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        ParseError {
            message: message.to_string(),
            source: Some(Box::new(source)),
        }
    }
}
