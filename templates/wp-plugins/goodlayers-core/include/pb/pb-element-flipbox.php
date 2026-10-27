<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('flipbox', 'gdlr_core_pb_element_flipbox'); 
	
	if( !class_exists('gdlr_core_pb_element_flipbox') ){
		class gdlr_core_pb_element_flipbox{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'icon_pencil-edit_alt',
					'title' => esc_html__('Flipbox', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return array(
					'f-general' => array(
						'title' => esc_html__('Main (front)', 'goodlayers-core'),
						'options' => array(
							'media-type' => array(
								'title' => esc_html__('Media Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'icon' => esc_html__('Icon', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core'),
								),
								'default' => 'icon'
							),
							'icon' => array(
								'title' => esc_html__('Icon', 'goodlayers-core'),
								'type' => 'icons',
								'allow-none' => true,
								'default' => 'fa fa-android',
								'condition' => array( 'media-type' => 'icon' ),
								'wrapper-class' => 'gdlr-core-fullsize' 
							),
							'image' => array(
								'title' => esc_html__('Upload', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'media-type' => 'image' )
							),
							'title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Flipbox Item Title', 'goodlayers-core'),
							),
							'title-divider' => array(
								'title' => esc_html__('Title Divider', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'caption' => array(
								'title' => esc_html__('Caption', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Flipbox Item Caption', 'goodlayers-core'),
							),
							'content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Flipbox item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'button-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'text'
							),
							'button-link' => array(
								'title' => esc_html__('Button Link', 'goodlayers-core'),
								'type' => 'text'
							),
							'link-to' => array(
								'title' => esc_html__('Link To', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'custom-url' => esc_html__('Custom Url', 'goodlayers-core'),
									'lb-custom-image' => esc_html__('Lightbox with custom image', 'goodlayers-core'),
									'lb-video' => esc_html__('Video Lightbox', 'goodlayers-core'),
								),
								'default' => 'custom-url'
							),
							'custom-image' => array(
								'title' => esc_html__('Upload Custom Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'link-to' => 'lb-custom-image' )
							),
							'video-url' => array(
								'title' => esc_html__('Video Url ( Youtube / Vimeo )', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'link-to' => 'lb-video' )
							),
							'link-url' => array(
								'title' => esc_html__('Item Link URL', 'goodlayers-core'),
								'type' => 'text',
								'default' => '#',
								'condition' => array( 'link-to' => 'custom-url' )
 							), 
							'link-target' => array(
								'title' => esc_html__('Item Link Target', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'_self' => esc_html__('Current Screen', 'goodlayers-core'),
									'_blank' => esc_html__('New Window', 'goodlayers-core'),
								),
								'default' => '_self',
								'condition' => array( 'link-to' => 'custom-url' )
							),						
						)
					), // General
					'f-style' => array(
						'title' => esc_html__('Style (front)', 'goodlayers-core'),
						'options' => array(
							'text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center',
							),			
							'icon-align' => array(
								'title' => esc_html__('Icon Align', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'top' => esc_html__('Top', 'goodlayers-core'),
									'left' => esc_html__('Left', 'goodlayers-core')
								),
								'condition' => array( 'text-align' => 'left' )
							),					
							'icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'title-divider-color' => array(
								'title' => esc_html__('Title Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'caption-color' => array(
								'title' => esc_html__('Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'button-color' => array(
								'title' => esc_html__('Button Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '35px'
							),
							'title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '22px'
							),
							'title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '700',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px'
							),
							'content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px'
							),
							'content-padding' => array(
								'title' => esc_html__('Content Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top' => '50px', 'right' => '40px', 'bottom' => '40px', 'left' => '40px', 'settings' => 'unlink' )
							),
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						)
					), // f-style
					'f-background' => array(
						'title' => esc_html__('BG Style (front)', 'goodlayers-core'),
						'options' => array(
							'sync-height' => array(
								'title' => esc_html__('Sync Height ( With different items )', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Use to sync the height among an items with the same keyword. The height will be fixed so be cautious not to use it on item with dynamic height.', 'goodlayers-core'),
							),
							'centering-sync-height-content' => array(
								'title' => esc_html__('Positioning Content In The Middle Of This Item ( When sync hieght is being used )', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							), 
							'background-image' => array(
								'title' => esc_html__('Background Image', 'goodlayers-core'),
								'type' => 'upload'
							),
							'background-opacity' => array(
								'title' => esc_html__('Background Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.62'
							),
							'border-type' => array(
								'title' => esc_html__('Border Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'outer' => esc_html__('Outer Border', 'goodlayers-core'),
									'inner' => esc_html__('Inner border', 'goodlayers-core'),
								),
								'default' => 'outer'
							),
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'border-type' => array('outer', 'inner') )
							),
							'border-width' => array(
								'title' => esc_html__('Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'5px', 'right'=>'5px', 'bottom'=>'5px', 'left'=>'5px', 'settings'=>'link' ),
								'condition' => array( 'border-type' => array('outer', 'inner') )
							),
							'border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '3px',
								'condition' => array( 'border-type' => array('outer', 'inner') )
							),
							'pre-border-space' => array(
								'title' => esc_html__('Spaces Before Border', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' ),
								'condition' => array( 'border-type' => 'inner' )
							),
							'shadow-size' => array(
								'title' => esc_html__('Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'shadow-color' => array(
								'title' => esc_html__('Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'shadow-opacity' => array(
								'title' => esc_html__('Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
						)
					), // f-background
					'b-general' => array(
						'title' => esc_html__('Main (back)', 'goodlayers-core'),
						'options' => array(
							'b-media-type' => array(
								'title' => esc_html__('Media Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'icon' => esc_html__('Icon', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core'),
								),
								'default' => 'icon'
							),
							'b-icon' => array(
								'title' => esc_html__('Icon', 'goodlayers-core'),
								'type' => 'icons',
								'allow-none' => true,
								'default' => 'fa fa-android',
								'condition' => array( 'b-media-type' => 'icon' ),
								'wrapper-class' => 'gdlr-core-fullsize' 
							),
							'b-image' => array(
								'title' => esc_html__('Upload', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'b-media-type' => 'image' )
							),
							'b-title' => array(
								'title' => esc_html__('Title', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Flipbox Item Title', 'goodlayers-core'),
							),
							'b-title-divider' => array(
								'title' => esc_html__('Title Divider', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'b-caption' => array(
								'title' => esc_html__('Caption', 'goodlayers-core'),
								'type' => 'text',
								'default' => esc_html__('Flipbox Item Caption', 'goodlayers-core'),
							),
							'b-content' => array(
								'title' => esc_html__('Content', 'goodlayers-core'),
								'type' => 'tinymce',
								'default' => esc_html__('Flipbox item sample content', 'goodlayers-core'),
								'wrapper-class' => 'gdlr-core-fullsize'
							),
							'b-button-text' => array(
								'title' => esc_html__('Button Text', 'goodlayers-core'),
								'type' => 'text'
							),
							'b-button-link' => array(
								'title' => esc_html__('Button Link', 'goodlayers-core'),
								'type' => 'text'
							)				
						)
					), // b-general
					'b-style' => array(
						'title' => esc_html__('Style (back)', 'goodlayers-core'),
						'options' => array(
							'b-text-align' => array(
								'title' => esc_html__('Text Align', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'text-align',
								'default' => 'center',
							),	
							'b-icon-align' => array(
								'title' => esc_html__('Icon Align', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'top' => esc_html__('Top', 'goodlayers-core'),
									'left' => esc_html__('Left', 'goodlayers-core')
								),
								'condition' => array( 'b-text-align' => 'left' )
							),				
							'b-icon-color' => array(
								'title' => esc_html__('Icon Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-title-color' => array(
								'title' => esc_html__('Title Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-title-divider-color' => array(
								'title' => esc_html__('Title Divider Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-caption-color' => array(
								'title' => esc_html__('Caption Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-content-color' => array(
								'title' => esc_html__('Content Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-button-color' => array(
								'title' => esc_html__('Button Background', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'b-icon-size' => array(
								'title' => esc_html__('Icon Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '35px'
							),
							'b-title-size' => array(
								'title' => esc_html__('Title Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '22px'
							),
							'b-title-font-weight' => array(
								'title' => esc_html__('Title Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'default' => '700',
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'b-caption-size' => array(
								'title' => esc_html__('Caption Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px'
							),
							'b-content-size' => array(
								'title' => esc_html__('Content Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'default' => '15px'
							),
							'b-content-padding' => array(
								'title' => esc_html__('Content Padding', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top' => '50px', 'right' => '40px', 'bottom' => '40px', 'left' => '40px', 'settings' => 'unlink' )
							),
						)
					), // b-style
					'b-background' => array(
						'title' => esc_html__('BG Style (back)', 'goodlayers-core'),
						'options' => array(
							'b-background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							), 
							'b-background-image' => array(
								'title' => esc_html__('Background Image', 'goodlayers-core'),
								'type' => 'upload'
							),
							'b-background-opacity' => array(
								'title' => esc_html__('Background Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.62'
							),
							'b-border-type' => array(
								'title' => esc_html__('Border Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'outer' => esc_html__('Outer Border', 'goodlayers-core'),
									'inner' => esc_html__('Inner border', 'goodlayers-core'),
								),
								'default' => 'outer'
							),
							'b-border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'b-border-type' => array('outer', 'inner') )
							),
							'b-border-width' => array(
								'title' => esc_html__('Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'5px', 'right'=>'5px', 'bottom'=>'5px', 'left'=>'5px', 'settings'=>'link' ),
								'condition' => array( 'b-border-type' => array('outer', 'inner') )
							),
							'b-border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '3px',
								'condition' => array( 'b-border-type' => array('outer', 'inner') )
							),
							'b-pre-border-space' => array(
								'title' => esc_html__('Spaces Before Border', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' ),
								'condition' => array( 'b-border-type' => 'inner' )
							),
						)
					) // b-background					
				);
			}
			
			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings);
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-flipbox-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	var flipbox_elem = jQuery('#gdlr-core-preview-flipbox-<?php echo esc_attr($id); ?>').parent().gdlr_core_flipbox();
	new gdlr_core_sync_height(flipbox_elem.closest('.gdlr-core-page-builder-body'));
});
</script><?php	
				$content .= ob_get_contents();
				ob_end_clean();
				
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array() ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'icon' => 'fa fa-android', 
						'title' => esc_html__('Flipbox Item Title', 'goodlayers-core'),
						'caption' => esc_html__('Flipbox Item Caption', 'goodlayers-core'),
						'content' => esc_html__('Flipbox item sample content', 'goodlayers-core'),
						'text-align' => 'center',
						
						'b-icon' => 'fa fa-android', 
						'b-title' => esc_html__('Flipbox Item Title', 'goodlayers-core'),
						'b-caption' => esc_html__('Flipbox Item Caption', 'goodlayers-core'),
						'b-content' => esc_html__('Flipbox item sample content', 'goodlayers-core'),
						'b-text-align' => 'center',
						
						'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				if( empty($settings['sync-height']) ){
					$id = mt_rand(0, 9999);
					$settings['sync-height'] = 'gdlr-core-flipbox-id-' . $id;
				}
				
				// start printing item
				$extra_class  = empty($settings['class'])? '': $settings['class'];
				$ret  = '<div class="gdlr-core-flipbox-item gdlr-core-item-pdlr gdlr-core-item-pdb ' . esc_attr($extra_class) . '" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';
				$ret .= '<div class="gdlr-core-flipbox gdlr-core-js" >';
				
				// flipbox front
				$settings['icon-size'] = (empty($settings['icon-size']) || $settings['icon-size'] == '35px')? '': $settings['icon-size'];
				$settings['title-size'] = (empty($settings['title-size']) || $settings['title-size'] == '22px')? '': $settings['title-size'];
				$settings['title-font-weight'] = (empty($settings['title-font-weight']) || $settings['title-font-weight'] == '700')? '': $settings['title-font-weight'];
				$settings['caption-size'] = (empty($settings['caption-size']) || $settings['caption-size'] == '15px')? '': $settings['caption-size'];
				$settings['content-size'] = (empty($settings['content-size']) || $settings['content-size'] == '15px')? '': $settings['content-size'];				
				$settings['border-type'] = empty($settings['border-type'])? 'outer': $settings['border-type'];
				$settings['content-padding'] =  (empty($settings['content-padding']) || $settings['content-padding'] == array(
						'top' => '50px', 'right' => '40px', 'bottom' => '40px', 'left' => '40px', 'settings' => 'unlink'
					))? '': $settings['content-padding'];
				$settings['border-width'] = (empty($settings['border-width']) || $settings['border-width'] == array( 
						'top'=>'5px', 'right'=>'5px', 'bottom'=>'5px', 'left'=>'5px', 'settings'=>'link' 
					))? '': $settings['border-width'];

				$front_wrap_class  = ' gdlr-core-' . (empty($settings['text-align'])? 'left': $settings['text-align']) . '-align';
				$front_wrap_class .= ' gdlr-core-icon-' . (empty($settings['icon-align'])? 'top': $settings['icon-align']) . '';
				$front_wrap_class .= ' gdlr-core-flipbox-type-' . $settings['border-type'];
				$front_wrap_attr = array( 
					'background-color'=>empty($settings['background-color'])? '': $settings['background-color'],
					'padding'=>$settings['content-padding'],
					'background-shadow-size'=>empty($settings['shadow-size'])? '': $settings['shadow-size'],
					'background-shadow-color'=>empty($settings['shadow-color'])? '': $settings['shadow-color'],
					'background-shadow-opacity'=>empty($settings['shadow-opacity'])? '': $settings['shadow-opacity'],
				);
				if( $settings['border-type'] == 'outer' ){
					$front_wrap_attr['border-width'] = $settings['border-width'];
					$front_wrap_attr['border-radius'] = empty($settings['border-radius'])? '': $settings['border-radius'];
					$front_wrap_attr['border-color'] = empty($settings['border-color'])? '': $settings['border-color'];
				}			
				$ret .= '<div class="gdlr-core-flipbox-front gdlr-core-js ' . esc_attr($front_wrap_class) . '" ';
				$ret .= gdlr_core_esc_style($front_wrap_attr) . ' ';
				if( !empty($settings['sync-height']) ){
					$ret .= ' data-sync-height="' . esc_attr($settings['sync-height']) . '" ';
					if( !empty($settings['centering-sync-height-content']) && $settings['centering-sync-height-content'] == 'enable' ){
						$ret .= ' data-sync-height-center';
					}
				}
				$ret .= ' >';
				if( !empty($settings['background-image']) ){
					$ret .= '<div class="gdlr-core-flipbox-background" ' . gdlr_core_esc_style(array(
						'background-image'=>$settings['background-image'],
						'opacity'=>(empty($settings['background-opacity']) || $settings['background-opacity']=='1')? '': $settings['background-opacity']
					)) . ' ></div>';
				}
				if( $settings['border-type'] == 'inner' ){
					$ret .= '<div class="gdlr-core-flipbox-frame" ' . gdlr_core_esc_style(array(
						'margin'=>(empty($settings['pre-border-space']) || $settings['pre-border-space'] == array( 
								'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link'
							))? '': $settings['pre-border-space'],
						'border-width'=>$settings['border-width'],
						'border-radius'=>(empty($settings['border-radius']) || $settings['b-border-radius'] == '3px')? '': $settings['border-radius'],
						'border-color'=>empty($settings['border-color'])? '': $settings['border-color']
					)) . ' ></div>';
				}
				$ret .= '<div class="gdlr-core-flipbox-content gdlr-core-sync-height-content" >';
				if( empty($settings['media-type']) || $settings['media-type'] == 'icon' ){
					if( !empty($settings['icon']) ){
						$ret .= '<i class="gdlr-core-flipbox-item-icon ' . esc_attr($settings['icon']) . '" ' . gdlr_core_esc_style(array(
							'font-size' => $settings['icon-size'],
							'color' => empty($settings['icon-color'])? '': $settings['icon-color']
						)) . ' ></i>';
					}
				}else{
					if( !empty($settings['image']) ){
						$ret .= '<div class="gdlr-core-flipbox-item-image gdlr-core-media-image" >';
						$ret .= gdlr_core_get_image($settings['image']);
						$ret .= '</div>';
					}
				}
				$ret .= '<div class="gdlr-core-flipbox-content-wrap" >';
				if( !empty($settings['title']) ){
					$ret .= '<h3 class="gdlr-core-flipbox-item-title" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['title-size'],
						'font-weight' => $settings['title-font-weight'],
						'color' => empty($settings['title-color'])? '': $settings['title-color']
					)) . ' >' . gdlr_core_text_filter($settings['title']) . '</h3>';
				}
				if( !empty($settings['title-divider']) && $settings['title-divider'] == 'enable' ){
					$ret .= '<div class="gdlr-core-flipbox-title-divider" ' . gdlr_core_esc_style(array(
						'border-color' => empty($settings['title-divider-color'])? '': $settings['title-divider-color']
					)) . ' ></div>';
				}
				if( !empty($settings['caption']) ){
					$ret .= '<div class="gdlr-core-flipbox-item-caption gdlr-core-title-font" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['caption-size'],
						'color' => empty($settings['caption-color'])? '': $settings['caption-color']
					)) . ' >' . gdlr_core_text_filter($settings['caption']) . '</div>';
				}
				if( !empty($settings['content']) ){
					$ret .= '<div class="gdlr-core-flipbox-item-content" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['content-size'],
						'color' => empty($settings['content-color'])? '': $settings['content-color']
					)) . '>' . gdlr_core_content_filter($settings['content']) . '</div>';
				}
				if( !empty($settings['button-text']) && !empty($settings['button-link']) ){
					$ret .= '<a class="gdlr-core-button gdlr-core-flipbox-item-button" href="' . esc_attr($settings['button-link']) . '" ' . gdlr_core_esc_style(array(
						'background-color' => empty($settings['button-color'])? '': $settings['button-color']
					)) . ' >' . gdlr_core_text_filter($settings['button-text']) . '</a>';
				}
				$ret .= '</div>'; // gdlr-core-flipbox-content-wrap
				$ret .= '</div>'; // gdlr-core-flipbox-content
				$ret .= '</div>'; // gdlr-core-flipbox-front
				
				// flipbox back
				$settings['b-icon-size'] = (empty($settings['b-icon-size']) || $settings['b-icon-size'] == '35px')? '': $settings['b-icon-size'];
				$settings['b-title-size'] = (empty($settings['b-title-size']) || $settings['b-title-size'] == '22px')? '': $settings['b-title-size'];
				$settings['b-title-font-weight'] = (empty($settings['b-title-font-weight']) || $settings['b-title-font-weight'] == '700')? '': $settings['b-title-font-weight'];
				$settings['b-caption-size'] = (empty($settings['b-caption-size']) || $settings['b-caption-size'] == '15px')? '': $settings['b-caption-size'];
				$settings['b-content-size'] = (empty($settings['b-content-size']) || $settings['b-content-size'] == '15px')? '': $settings['b-content-size'];				
				$settings['b-border-type'] = empty($settings['b-border-type'])? 'outer': $settings['b-border-type'];
				$settings['b-content-padding'] =  (empty($settings['b-content-padding']) || $settings['b-content-padding'] == array(
						'top' => '50px', 'right' => '40px', 'bottom' => '40px', 'left' => '40px', 'settings' => 'unlink'
					))? '': $settings['b-content-padding'];
				$settings['b-border-width'] = (empty($settings['b-border-width']) || $settings['b-border-width'] == array( 
						'top'=>'5px', 'right'=>'5px', 'bottom'=>'5px', 'left'=>'5px', 'settings'=>'link' 
					))? '': $settings['b-border-width'];

				$back_wrap_class  = ' gdlr-core-' . (empty($settings['b-text-align'])? 'left': $settings['b-text-align']) . '-align';
				$back_wrap_class .= ' gdlr-core-icon-' . (empty($settings['b-icon-align'])? 'top': $settings['b-icon-align']) . '';
				$back_wrap_class .= ' gdlr-core-flipbox-type-' . $settings['b-border-type'];
				$back_wrap_attr = array( 
					'background-color'=>empty($settings['b-background-color'])? '': $settings['b-background-color'],
					'padding' => $settings['b-content-padding'],
					'background-shadow-size'=>empty($settings['shadow-size'])? '': $settings['shadow-size'],
					'background-shadow-color'=>empty($settings['shadow-color'])? '': $settings['shadow-color'],
					'background-shadow-opacity'=>empty($settings['shadow-opacity'])? '': $settings['shadow-opacity'],
				);
				if( $settings['b-border-type'] == 'outer' ){
					$back_wrap_attr['border-width'] = $settings['b-border-width'];
					$back_wrap_attr['border-radius'] = (empty($settings['b-border-radius']) || $settings['b-border-radius'] == '3px')? '': $settings['b-border-radius'];
					$back_wrap_attr['border-color'] = empty($settings['b-border-color'])? '': $settings['b-border-color'];
				}
				$ret .= '<div class="gdlr-core-flipbox-back gdlr-core-js ' . esc_attr($back_wrap_class) . '" ';
				$ret .= gdlr_core_esc_style($back_wrap_attr) . ' ';
				if( !empty($settings['sync-height']) ){
					$ret .= ' data-sync-height="' . esc_attr($settings['sync-height']) . '" ';
					if( !empty($settings['centering-sync-height-content']) && $settings['centering-sync-height-content'] == 'enable' ){
						$ret .= ' data-sync-height-center';
					}
				}
				$ret .= ' >';
				if( !empty($settings['b-background-image']) ){
					$ret .= '<div class="gdlr-core-flipbox-background" ' . gdlr_core_esc_style(array(
						'background-image'=>$settings['b-background-image'],
						'opacity'=>(empty($settings['b-background-opacity']) || $settings['b-background-opacity']=='1')? '': $settings['b-background-opacity']
					)) . ' ></div>';
				}
				if( $settings['b-border-type'] == 'inner' ){
					$ret .= '<div class="gdlr-core-flipbox-frame" ' . gdlr_core_esc_style(array(
						'margin'=>(empty($settings['b-pre-border-space']) || $settings['b-pre-border-space'] == array( 
								'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link'
							))? '': $settings['b-pre-border-space'],
						'border-width'=>$settings['b-border-width'],
						'border-radius'=>empty($settings['b-border-radius'])? '': $settings['b-border-radius'],
						'border-color'=>empty($settings['b-border-color'])? '': $settings['b-border-color'],
					)) . '></div>';
				}
				$ret .= '<div class="gdlr-core-flipbox-content gdlr-core-sync-height-content" >';
				if( empty($settings['b-media-type']) || $settings['b-media-type'] == 'icon' ){
					if( !empty($settings['b-icon']) ){
						$ret .= '<i class="gdlr-core-flipbox-item-icon ' . esc_attr($settings['b-icon']) . '" ' . gdlr_core_esc_style(array(
							'font-size' => $settings['b-icon-size'],
							'color' => empty($settings['b-icon-color'])? '': $settings['b-icon-color']
						)) . ' ></i>';
					}
				}else{
					if( !empty($settings['b-image']) ){
						$ret .= '<div class="gdlr-core-flipbox-item-image gdlr-core-media-image" >';
						$ret .= gdlr_core_get_image($settings['b-image']);
						$ret .= '</div>';
					}
				}

				$ret .= '<div class="gdlr-core-flipbox-content-wrap " >';
				if( !empty($settings['b-title']) ){
					$ret .= '<h3 class="gdlr-core-flipbox-item-title" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['b-title-size'],
						'font-weight' => $settings['b-title-font-weight'],
						'color' => empty($settings['b-title-color'])? '': $settings['b-title-color']
					)) . ' >' . gdlr_core_text_filter($settings['b-title']) . '</h3>';
				}
				if( !empty($settings['b-title-divider']) && $settings['b-title-divider'] == 'enable' ){
					$ret .= '<div class="gdlr-core-flipbox-title-divider" ' . gdlr_core_esc_style(array(
						'border-color' => empty($settings['b-title-divider-color'])? '': $settings['b-title-divider-color']
					)) . ' ></div>';
				}
				if( !empty($settings['b-caption']) ){
					$ret .= '<div class="gdlr-core-flipbox-item-caption gdlr-core-title-font" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['b-caption-size'],
						'color' => empty($settings['b-caption-color'])? '': $settings['b-caption-color']
					)) . ' >' . gdlr_core_text_filter($settings['b-caption']) . '</div>';
				}
				if( !empty($settings['b-content']) ){
					$ret .= '<div class="gdlr-core-flipbox-item-content" ' . gdlr_core_esc_style(array(
						'font-size' => $settings['b-content-size'],
						'color' => empty($settings['b-content-color'])? '': $settings['b-content-color']
					)) . '>' . gdlr_core_content_filter($settings['b-content']) . '</div>';
				}
				if( !empty($settings['b-button-text']) && !empty($settings['b-button-link']) ){
					$ret .= '<a class="gdlr-core-button gdlr-core-flipbox-item-button" href="' . esc_attr($settings['b-button-link']) . '" ' . gdlr_core_esc_style(array(
						'background-color' => empty($settings['b-button-color'])? '': $settings['b-button-color']
					)) . ' >' . gdlr_core_text_filter($settings['b-button-text']) . '</a>';
				}

				if( empty($settings['link-to']) || $settings['link-to'] == 'custom-url' ){
					if( !empty($settings['link-url']) && !empty($settings['link-target']) ){
						$ret .= '<a class="gdlr-core-flipbox-link" href="' . esc_url($settings['link-url']) . '" target="' . esc_attr($settings['link-target']) . '" ></a>';
					}
				}else if( $settings['link-to'] == 'lb-custom-image' ){
					$image_url = '';
					$caption = '';
					if( !empty($settings['custom-image']) ){
						if( is_numeric($settings['custom-image']) ){
							$image_url = gdlr_core_get_image_url($settings['custom-image']);
							$caption = gdlr_core_get_image_info($settings['custom-image'], 'caption');;
						}else{
							$image_url = $settings['custom-image'];
						}
					}

					$ret .= '<a ' . gdlr_core_get_lightbox_atts(array(
						'class'=>'gdlr-core-flipbox-link ',
						'url'=>$image_url,
						'captin'=>$caption
					)) . ' ></a>';
				}else if( $settings['link-to'] == 'lb-video' ){
					$ret .= '<a ' . gdlr_core_get_lightbox_atts(array(
						'class'=>'gdlr-core-flipbox-link ',
						'url'=>$settings['video-url'],
						'type'=>'video'
					)) . ' ></a>';
				}				
				$ret .= '</div>'; // gdlr-core-flipbox-content-wrap
				$ret .= '</div>'; // gdlr-core-flipbox-content
				$ret .= '</div>'; // gdlr-core-flipbox-back
				
				$ret .= '</div>'; // gdlr-core-flipbox
				$ret .= '</div>';
				
				return $ret;
			}
			
		} // gdlr_core_pb_element_flipbox
	} // class_exists	