require ["fileinto", "reject", "variables", "copy"];

if address :matches :all ["To", "Cc"] "lists+*@example.com"
{
    fileinto :copy "lists/${1}";
} 

