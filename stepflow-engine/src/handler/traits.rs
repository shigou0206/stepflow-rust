use async_trait::async_trait;
use serde_json::Value;
use super::context::{StateExecutionContext, StateExecutionResult};

/// 统一的状态处理器接口
#[async_trait]
pub trait StateHandler: Send + Sync {
    /// 处理状态逻辑
    async fn handle(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String>;

    /// 获取状态类型
    fn state_type(&self) -> &'static str;

    /// 执行状态，包含统一的生命周期管理
    async fn execute(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        // 1. 创建状态记录
        ctx.create_state_record(input).await?;
        
        // 2. 发送进入事件
        ctx.dispatch_enter(input).await;
        
        // 3. 执行具体处理逻辑
        match self.handle(ctx, input).await {
            Ok(result) => {
                // 4. 更新成功状态
                ctx.update_success_state(&result.output).await?;
                
                // 5. 发送成功事件
                ctx.dispatch_success(&result.output).await;
                
                Ok(result)
            }
            Err(e) => {
                // 4. 更新失败状态
                ctx.update_failure_state(&e).await?;
                
                // 5. 发送失败事件
                ctx.dispatch_failure(&e).await;
                
                Err(e)
            }
        }
    }
} 