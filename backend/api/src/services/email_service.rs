use anyhow::Result;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::AppState;

pub struct EmailService<'a> {
    state: &'a AppState,
}

impl<'a> EmailService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    fn build_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(
            self.state.config.smtp_user.clone(),
            self.state.config.smtp_password.clone(),
        );

        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.state.config.smtp_host)?
            .port(self.state.config.smtp_port)
            .credentials(creds)
            .build();

        Ok(transport)
    }

    async fn send(&self, to: &str, subject: &str, html_body: &str) -> Result<()> {
        let from = format!(
            "{} <{}>",
            self.state.config.smtp_from_name,
            self.state.config.smtp_from_email
        );

        let email = Message::builder()
            .from(from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())?;

        let transport = self.build_transport()?;
        transport.send(email).await?;
        Ok(())
    }

    pub async fn send_verification_email(
        &self,
        to: &str,
        name: &str,
        token: &str,
    ) -> Result<()> {
        let verify_url = format!(
            "{}/api/v1/auth/verify-email?token={}",
            self.state.config.app_url,
            token
        );

        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="background: linear-gradient(135deg, #1e293b, #0f172a); padding: 30px; border-radius: 12px; text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #22c55e; margin: 0; font-size: 28px;">📱 WhatsUp Platform</h1>
                </div>
                <h2 style="color: #1e293b;">Hello {name},</h2>
                <p style="color: #64748b; line-height: 1.6;">
                    Thanks for signing up! Please verify your email address to activate your account.
                </p>
                <div style="text-align: center; margin: 30px 0;">
                    <a href="{verify_url}"
                       style="background: #22c55e; color: white; padding: 14px 32px; border-radius: 8px; text-decoration: none; font-weight: bold; font-size: 16px;">
                        Verify Email Address
                    </a>
                </div>
                <p style="color: #94a3b8; font-size: 13px;">This link expires in 24 hours.</p>
            </body>
            </html>
            "#,
            name = name,
            verify_url = verify_url
        );

        self.send(to, "Verify your WhatsUp account", &html).await
    }

    pub async fn send_password_reset_email(
        &self,
        to: &str,
        name: &str,
        token: &str,
    ) -> Result<()> {
        let reset_url = format!(
            "{}/reset-password?token={}",
            self.state.config.frontend_url,
            token
        );

        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="background: linear-gradient(135deg, #1e293b, #0f172a); padding: 30px; border-radius: 12px; text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #22c55e; margin: 0;">📱 WhatsUp Platform</h1>
                </div>
                <h2 style="color: #1e293b;">Password Reset Request</h2>
                <p style="color: #64748b;">Hi {name},<br>We received a request to reset your password.</p>
                <div style="text-align: center; margin: 30px 0;">
                    <a href="{reset_url}"
                       style="background: #f59e0b; color: white; padding: 14px 32px; border-radius: 8px; text-decoration: none; font-weight: bold;">
                        Reset Password
                    </a>
                </div>
                <p style="color: #94a3b8; font-size: 13px;">This link expires in 2 hours. If you didn't request this, ignore this email.</p>
            </body>
            </html>
            "#,
            name = name,
            reset_url = reset_url
        );

        self.send(to, "Reset your WhatsUp password", &html).await
    }

    pub async fn send_invitation_email(
        &self,
        to: &str,
        org_name: &str,
        invited_by: &str,
        token: &str,
    ) -> Result<()> {
        let invite_url = format!(
            "{}/accept-invite?token={}",
            self.state.config.frontend_url,
            token
        );

        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
                <div style="background: linear-gradient(135deg, #1e293b, #0f172a); padding: 30px; border-radius: 12px; text-align: center; margin-bottom: 30px;">
                    <h1 style="color: #22c55e; margin: 0;">📱 WhatsUp Platform</h1>
                </div>
                <h2 style="color: #1e293b;">You're invited!</h2>
                <p style="color: #64748b; line-height: 1.6;">
                    <strong>{invited_by}</strong> has invited you to join <strong>{org_name}</strong> on WhatsUp Platform.
                </p>
                <div style="text-align: center; margin: 30px 0;">
                    <a href="{invite_url}"
                       style="background: #6366f1; color: white; padding: 14px 32px; border-radius: 8px; text-decoration: none; font-weight: bold;">
                        Accept Invitation
                    </a>
                </div>
                <p style="color: #94a3b8; font-size: 13px;">This invitation expires in 7 days.</p>
            </body>
            </html>
            "#,
            invited_by = invited_by,
            org_name = org_name,
            invite_url = invite_url
        );

        self.send(to, &format!("You're invited to join {} on WhatsUp", org_name), &html).await
    }
}
