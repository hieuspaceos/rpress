<?php 

    add_action('init', 'gdlr_core_product_settings');
    if( !function_exists('gdlr_core_product_settings') ){
        function gdlr_core_product_settings(){
            if( class_exists('gdlr_core_taxonomy_option') ){
                new gdlr_core_taxonomy_option(array(
                    'taxonomy' => 'product_cat',
                    'options' => array(
                        'icon' => array(
                            'title' => esc_html__('Icon (For product item category filter)', 'goodlayers-core'),
                            'type' => 'icons',
                            'allow-none' => true,
                            'default' => 'none',
                            'wrapper-class' => 'gdlr-core-fullsize'
                        )
                    )
                ));
            }
        }
    }   