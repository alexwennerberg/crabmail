require ["fileinto", "reject", "variables", "copy"];

if address :matches :all "to" "lists+*@example.com"
{
    fileinto :copy "lists/${1}";
} 

