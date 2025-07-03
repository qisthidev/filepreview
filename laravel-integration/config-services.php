<?php

/*
|--------------------------------------------------------------------------
| Third Party Services Configuration
|--------------------------------------------------------------------------
|
| Add this configuration to your config/services.php file in Laravel
|
*/

return [

    // ... existing services configuration ...

    /*
    |--------------------------------------------------------------------------
    | File Preview Service
    |--------------------------------------------------------------------------
    |
    | Configuration for the Rust-based file preview microservice.
    | This service handles generating previews for various file formats.
    |
    */

    'filepreview' => [
        'url' => env('FILEPREVIEW_SERVICE_URL', 'http://localhost:3000'),
        'timeout' => env('FILEPREVIEW_TIMEOUT', 30),
        'connect_timeout' => env('FILEPREVIEW_CONNECT_TIMEOUT', 10),
        'max_file_size' => env('FILEPREVIEW_MAX_FILE_SIZE', 52428800), // 50MB in bytes
        'default_options' => [
            'width' => env('FILEPREVIEW_DEFAULT_WIDTH', null),
            'height' => env('FILEPREVIEW_DEFAULT_HEIGHT', null),
            'quality' => env('FILEPREVIEW_DEFAULT_QUALITY', 85),
        ],
    ],

    // ... rest of services configuration ...

];