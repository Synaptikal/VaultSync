# Email Configuration Guide

VaultSync supports sending emails for receipts, notifications, and alerts. This guide covers how to configure email using your existing business email account.

---

## Quick Start

Add these environment variables to your `.env` file:

```env
# Gmail Example
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=yourshop@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=yourshop@gmail.com

# Outlook/Microsoft 365 Example
SMTP_HOST=smtp.office365.com
SMTP_PORT=587
SMTP_USERNAME=yourshop@outlook.com
SMTP_PASSWORD=your-password
SMTP_FROM=yourshop@outlook.com
```

---

## Gmail SMTP Setup

### Step 1: Enable 2-Factor Authentication
1. Go to [Google Account Security](https://myaccount.google.com/security)
2. Click on **2-Step Verification**
3. Follow the prompts to enable 2FA

> ⚠️ **Required**: Gmail requires 2FA to generate App Passwords

### Step 2: Generate an App Password
1. Go to [Google App Passwords](https://myaccount.google.com/apppasswords)
2. Select **Mail** as the app
3. Select **Windows Computer** (or your device type)
4. Click **Generate**
5. Copy the 16-character password (you won't see it again!)

### Step 3: Configure VaultSync
Add to your `.env` file:

```env
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=yourshop@gmail.com
SMTP_PASSWORD=xxxx xxxx xxxx xxxx
SMTP_FROM=yourshop@gmail.com
```

### Gmail Limits
| Account Type | Daily Limit |
|-------------|-------------|
| Free Gmail | 500 emails/day |
| Google Workspace | 2,000 emails/day |

---

## Outlook / Microsoft 365 SMTP Setup

### For Personal Outlook.com Accounts

```env
SMTP_HOST=smtp-mail.outlook.com
SMTP_PORT=587
SMTP_USERNAME=yourshop@outlook.com
SMTP_PASSWORD=your-account-password
SMTP_FROM=yourshop@outlook.com
```

### For Microsoft 365 Business Accounts

```env
SMTP_HOST=smtp.office365.com
SMTP_PORT=587
SMTP_USERNAME=yourshop@yourdomain.com
SMTP_PASSWORD=your-password
SMTP_FROM=yourshop@yourdomain.com
```

### Enabling SMTP in Microsoft 365 Admin

If using Microsoft 365 for business, you may need to enable SMTP AUTH:

1. Go to [Microsoft 365 Admin Center](https://admin.microsoft.com)
2. Navigate to **Users** → **Active Users**
3. Select the user account
4. Click **Mail** → **Manage email apps**
5. Enable **Authenticated SMTP**
6. Save changes

### Outlook Limits
| Account Type | Daily Limit |
|-------------|-------------|
| Outlook.com (free) | 300 emails/day |
| Microsoft 365 Business | 10,000 emails/day |

---

## Yahoo Mail SMTP Setup

```env
SMTP_HOST=smtp.mail.yahoo.com
SMTP_PORT=587
SMTP_USERNAME=yourshop@yahoo.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=yourshop@yahoo.com
```

Generate an app password at: [Yahoo Account Security](https://login.yahoo.com/account/security)

---

## GoDaddy Workspace Email

```env
SMTP_HOST=smtpout.secureserver.net
SMTP_PORT=587
SMTP_USERNAME=yourshop@yourdomain.com
SMTP_PASSWORD=your-password
SMTP_FROM=yourshop@yourdomain.com
```

---

## Zoho Mail

```env
SMTP_HOST=smtp.zoho.com
SMTP_PORT=587
SMTP_USERNAME=yourshop@yourdomain.com
SMTP_PASSWORD=your-password
SMTP_FROM=yourshop@yourdomain.com
```

---

## Testing Your Configuration

After configuring your `.env` file:

1. Restart VaultSync
2. Create a test transaction
3. Use the email receipt feature: `POST /api/transactions/{id}/email-receipt`

Check the application logs for any SMTP errors.

---

## Troubleshooting

### "Authentication failed"
- **Gmail**: Ensure you're using an App Password, not your regular password
- **Outlook 365**: Ensure SMTP AUTH is enabled for your account
- Double-check username and password for typos

### "Connection refused" or timeout
- Verify your firewall isn't blocking port 587
- Some networks block outbound SMTP - try from a different network
- Ensure SMTP_HOST is correct

### "Sender address rejected"
- The `SMTP_FROM` address must match your authenticated account
- Some providers don't allow sending from arbitrary addresses

### Emails going to spam
- Add SPF records to your domain DNS (if using a custom domain)
- Don't send too many emails too quickly
- Ensure email content isn't "spammy"

### Gmail "Less secure app access" error
- Google no longer supports "less secure apps"
- You **must** use an App Password with 2FA enabled

---

## Transactional Email Services (Alternative)

For higher volume or better deliverability, consider these services:

| Service | Free Tier | Notes |
|---------|-----------|-------|
| [SendGrid](https://sendgrid.com) | 100/day | Best free tier |
| [Mailgun](https://mailgun.com) | 5,000/month (3 months) | Good API |
| [Postmark](https://postmarkapp.com) | 100/month | Best deliverability |
| [Amazon SES](https://aws.amazon.com/ses/) | 62,000/month (from EC2) | Cheapest at scale |

To use these, configure their SMTP credentials in the same way shown above.

---

## Security Best Practices

1. **Never commit `.env` files** to version control
2. **Use App Passwords** instead of your main account password
3. **Consider a dedicated email** for VaultSync (e.g., `notifications@yourshop.com`)
4. **Monitor your sent folder** for unexpected emails (could indicate compromise)
5. **Rotate passwords** periodically

---

## Example Complete Configuration

```env
# Email Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=vaultsync-notifications@yourshop.com
SMTP_PASSWORD=abcd efgh ijkl mnop
SMTP_FROM=VaultSync Notifications <notifications@yourshop.com>

# SMS Configuration (Optional)
TWILIO_ACCOUNT_SID=ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
TWILIO_AUTH_TOKEN=your-auth-token
TWILIO_FROM_NUMBER=+15551234567
```

---

## Support

If you continue to have issues:
1. Check the VaultSync logs for detailed error messages
2. Test your SMTP credentials with an email client first
3. Contact your email provider's support for account-specific issues
